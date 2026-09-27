"""What is published: voices named by the turns as they stand."""

from __future__ import annotations

from clipmill.ipc.v1 import daemon_pb2
from clipmill_worker_speakers import _document
from clipmill_worker_speakers.voices import Clustering, Turn

AUDIO = "sha256:" + "2" * 64
VAD = "sha256:" + "3" * 64
SOURCE = "sha256:" + "1" * 64
MODEL = "sha256:" + "4" * 64


def test_voices_are_named_in_the_order_the_published_turns_are_heard() -> None:
    payload = daemon_pb2.SpeechStagePayloadV1(
        key_version="clipmill.speech-stage.v1", stage="speech-speakers"
    )
    # Voice 1 was first heard in a window smoothing gave to voice 0, so the
    # first turn is voice 2's: it is spk_1.
    turns = [
        Turn(0, 16_000, 2),
        Turn(16_000, 48_000, 0),
        Turn(48_000, 48_004, 1),
        Turn(64_000, 80_000, 2),
    ]
    document = _document(payload, AUDIO, VAD, SOURCE, 900_000, MODEL, Clustering(), turns)
    assert [
        (turn.speaker_id.root, turn.start_ticks, turn.end_ticks) for turn in document.turns
    ] == [
        ("spk_1", 0, 90_000),
        ("spk_2", 90_000, 270_000),
        ("spk_3", 270_000, 270_022),
        ("spk_1", 360_000, 450_000),
    ]
    assert [
        (speaker.speaker_id.root, speaker.speech_ticks, speaker.first_ticks)
        for speaker in document.speakers
    ] == [("spk_1", 180_000, 0), ("spk_2", 180_000, 90_000), ("spk_3", 22, 270_000)]
    assert document.clustering.window_ticks == 135_000
    assert document.clustering.hop_ticks == 67_500
    assert document.producer.implementation == "clipmill-worker-speakers@0.1.0+campplus-voxceleb"
    assert document.source_fingerprint.root == SOURCE


def test_a_turn_with_no_length_is_not_published() -> None:
    payload = daemon_pb2.SpeechStagePayloadV1(
        key_version="clipmill.speech-stage.v1", stage="speech-speakers"
    )
    document = _document(payload, AUDIO, VAD, SOURCE, 900_000, MODEL, Clustering(), [Turn(8, 8, 0)])
    assert document.turns == []
    assert document.speakers == []
