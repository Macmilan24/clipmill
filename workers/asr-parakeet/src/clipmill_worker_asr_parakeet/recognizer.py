"""Greedy Parakeet TDT decoding with native acoustic durations on the CPU."""

from __future__ import annotations

from collections.abc import Callable
from dataclasses import dataclass

import numpy as np
import onnxruntime as ort
from clipmill_worker_sdk.confidence import distribution
from clipmill_worker_sdk.gen.schemas.speech_asr import Confidence, Token, TransducerWord
from clipmill_worker_sdk.weights import VerifiedModel

# Ten-millisecond features, subsampled eightfold by the FastConformer encoder.
FRAME_TICKS = 7_200
SAMPLE_RATE = 16_000
MAX_SYMBOLS_PER_FRAME = 10


@dataclass(frozen=True)
class Emission:
    piece: str
    frame: int
    frames: int
    probability: float


def words_from_emissions(
    emissions: list[Emission], start_ticks: int, end_ticks: int
) -> tuple[list[TransducerWord], list[Token]]:
    """Keep punctuation with the spoken word and stop intervals before silent frames."""
    words: list[TransducerWord] = []
    tokens: list[Token] = []
    pieces: list[Emission] = []

    def flush() -> None:
        if not pieces:
            return
        text = "".join(piece.piece.replace("▁", "") for piece in pieces)
        if text:
            first, last = pieces[0], pieces[-1]
            p50, p10 = distribution([piece.probability for piece in pieces])
            words.append(
                TransducerWord(
                    text=text,
                    start_ticks=min(end_ticks, start_ticks + first.frame * FRAME_TICKS),
                    end_ticks=min(
                        end_ticks, start_ticks + (last.frame + last.frames) * FRAME_TICKS
                    ),
                    confidence=Confidence(p50=round(p50, 4), p10=round(p10, 4)),
                )
            )
        pieces.clear()

    for emission in emissions:
        piece = emission.piece
        if piece.startswith("<") and piece.endswith(">"):
            continue
        if piece.startswith("▁"):
            flush()
        if piece == "▁":
            continue
        tokens.append(
            Token(text=piece.replace("▁", " "), confidence=round(emission.probability, 4))
        )
        pieces.append(emission)
    flush()
    return words, tokens


class ParakeetRecognizer:
    """All sessions consume verified local files and share one declared CPU thread."""

    def __init__(self, model: VerifiedModel) -> None:
        ort.disable_telemetry_events()
        options = ort.SessionOptions()
        options.intra_op_num_threads = 1
        options.inter_op_num_threads = 1
        options.execution_mode = ort.ExecutionMode.ORT_SEQUENTIAL

        def session(name: str) -> ort.InferenceSession:
            return ort.InferenceSession(
                str(model.path(name)), sess_options=options, providers=["CPUExecutionProvider"]
            )

        self.features = session("nemo128.onnx")
        self.encoder = session("encoder-model.int8.onnx")
        self.decoder = session("decoder_joint-model.int8.onnx")
        self.vocabulary = {
            int(index): piece
            for line in model.path("vocab.txt").read_text(encoding="utf-8").splitlines()
            for piece, index in [line.rsplit(" ", 1)]
        }
        self.blank = next(i for i, token in self.vocabulary.items() if token == "<blk>")
        shapes = {entry.name: entry.shape for entry in self.decoder.get_inputs()}
        self.state_shape = (shapes["input_states_1"][0], 1, shapes["input_states_1"][2])

    def decode(
        self, audio: np.ndarray, *, check_cancelled: Callable[[], None] = lambda: None
    ) -> list[Emission]:
        if not audio.size:
            return []
        # The feature graph uses reflection padding. Very short VAD tails still
        # need enough samples for it, but emitted intervals are clipped to the tail.
        waveform = audio if audio.size >= 512 else np.pad(audio, (0, 512 - audio.size))
        samples = waveform.astype(np.float32, copy=False)[None, :]
        sample_counts = np.array([samples.shape[1]], dtype=np.int64)
        wave_input, length_input = self.features.get_inputs()
        features, lengths = self.features.run(
            None, {wave_input.name: samples, length_input.name: sample_counts}
        )
        check_cancelled()
        acoustics, length = self.encoder.run(
            ["outputs", "encoded_lengths"], {"audio_signal": features, "length": lengths}
        )
        frame_count = min(int(length[0]), acoustics.shape[2])
        states = [np.zeros(self.state_shape, dtype=np.float32) for _ in range(2)]
        previous = self.blank
        emissions: list[Emission] = []
        frame = 0
        symbols = 0
        while frame < frame_count:
            check_cancelled()
            logits, next_hidden, next_cell = self.decoder.run(
                ["outputs", "output_states_1", "output_states_2"],
                {
                    "encoder_outputs": acoustics[:, :, frame : frame + 1],
                    "targets": np.array([[previous]], dtype=np.int32),
                    "target_length": np.array([1], dtype=np.int32),
                    "input_states_1": states[0],
                    "input_states_2": states[1],
                },
            )
            scores = logits.reshape(-1)
            token_scores = scores[: len(self.vocabulary)]
            token = int(np.argmax(token_scores))
            # This checkpoint's duration classes are zero through four frames.
            duration = int(np.argmax(scores[len(self.vocabulary) :]))
            if token != self.blank:
                states = [next_hidden, next_cell]
                previous = token
                confidence = 1.0 / float(np.exp(token_scores - token_scores[token]).sum())
                emissions.append(Emission(self.vocabulary[token], frame, duration, confidence))
                symbols += 1
            if duration or token == self.blank or symbols >= MAX_SYMBOLS_PER_FRAME:
                frame += max(1, duration)
                symbols = 0
        return emissions
