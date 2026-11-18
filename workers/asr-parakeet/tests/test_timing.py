"""Duration evidence and the contract handed to transcript assembly."""

from pathlib import Path

import pytest
from clipmill_worker_asr_parakeet import CAPABILITIES, IMPLEMENTATION, timing
from clipmill_worker_asr_parakeet.recognizer import FRAME_TICKS, Emission, words_from_emissions
from clipmill_worker_sdk import DeterministicTaskError
from clipmill_worker_sdk.gen.schemas.speech_asr import SpeechAsr

ASR_ID = "sha256:" + "5" * 64
FIXTURE = Path(__file__).resolve().parents[3] / "contracts/fixtures/speech.asr/valid/tdt_words.json"


def test_only_speech_and_its_duration_projection_are_served() -> None:
    assert CAPABILITIES == ("speech-asr", "speech-tdt-timing")


def test_word_edges_follow_the_duration_head_and_silence_stays_out() -> None:
    start, end = 1000, 1000 + 14 * FRAME_TICKS
    words, tokens = words_from_emissions(
        [
            Emission("<|nospeech|>", 0, 4, 1),
            Emission("▁ret", 4, 1, 0.91),
            Emission("ake", 5, 2, 0.93),
            Emission(",", 7, 0, 0.8),
            Emission("▁sí", 12, 4, 0.99),
        ],
        start,
        end,
    )
    assert [w.text for w in words] == ["retake,", "sí"]
    assert [(w.start_ticks, w.end_ticks) for w in words] == [
        (start + 4 * FRAME_TICKS, start + 7 * FRAME_TICKS),
        (start + 12 * FRAME_TICKS, end),
    ]
    assert tokens[0].text == " ret" and tokens[-1].confidence == 0.99


def test_a_zero_duration_word_is_preserved_for_interpolation_by_assembly() -> None:
    words, _ = words_from_emissions([Emission("▁I", 0, 0, 0.95)], 5, 9005)
    assert words[0].start_ticks == words[0].end_ticks == 5


def test_timing_document_links_the_asr_artifact_and_preserves_word_order() -> None:
    asr = SpeechAsr.model_validate_json(FIXTURE.read_bytes())
    output = timing(asr, ASR_ID)
    assert output.frame_ticks == FRAME_TICKS
    assert output.asr_artifact_id.root == ASR_ID
    assert [w.text for w in output.words] == ["That", "works."]
    assert output.words[1].end_ticks == 4 * FRAME_TICKS
    assert output.coverage.aligned_ticks == 4 * FRAME_TICKS


@pytest.mark.parametrize("defect", ["wrong-producer", "foreign-word", "overlap", "out-of-window"])
def test_inconsistent_timing_is_rejected(defect: str) -> None:
    recognized = SpeechAsr.model_validate_json(FIXTURE.read_bytes())
    assert recognized.producer.implementation == IMPLEMENTATION
    second = recognized.segments[0].transducer_words[1]
    if defect == "wrong-producer":
        recognized.producer.implementation = "whisper"
    elif defect == "foreign-word":
        second.text = "else"
    elif defect == "overlap":
        second.start_ticks = 0
    else:
        second.end_ticks = recognized.segments[0].hint_end_ticks + 1
    with pytest.raises(DeterministicTaskError):
        timing(recognized, ASR_ID)
