from types import SimpleNamespace

from clipmill.ipc.v1 import daemon_pb2
from clipmill_worker_audio_events import KEY_VERSION, LEGACY_KEY_VERSION, STAGE, build_document
from clipmill_worker_audio_events.events import THRESHOLDS, Delivery, Event

DIGEST = "sha256:" + "2" * 64
AUDIO = "sha256:" + "b" * 64
SOURCE = "sha256:" + "a" * 64


def test_the_document_times_events_in_ticks_and_says_what_was_heard():
    payload = daemon_pb2.AudioEventsStagePayloadV1(
        key_version=KEY_VERSION, stage=STAGE, source_fingerprint=SOURCE
    )
    audio = SimpleNamespace(
        sample_rate=16_000,
        sample_count=16_000 * 30,
        duration_ticks=90_000 * 30,
        source_fingerprint=SOURCE,
    )
    model = SimpleNamespace(digest=DIGEST)
    found = [Event(kind="laughter", start_sample=48_000, end_sample=88_001, peak=0.8, mean=0.5)]

    document = build_document(payload, AUDIO, audio, model, found)

    assert document.detection.window_ticks == 180_000
    assert document.detection.hop_ticks == 90_000
    assert [entry.kind.value for entry in document.detection.thresholds] == list(THRESHOLDS)
    assert document.coverage.analyzed
    assert document.coverage.end_ticks == 2_700_000
    (event,) = document.events
    assert (event.start_ticks, event.end_ticks) == (270_000, 495_006)

    found.append(Event("emphasis", 90_000, 96_000, 0.97, 0.93, Delivery(180, -17, 4.1, 6)))
    updated = build_document(payload, AUDIO, audio, model, found, vad_artifact_id=DIGEST)
    assert updated.vad_artifact_id.root == DIGEST
    assert updated.events[1].delivery.level_change_db == 6
    payload.key_version = LEGACY_KEY_VERSION
    legacy = build_document(payload, AUDIO, audio, model, found[:1])
    assert legacy.vad_artifact_id is None
    assert legacy.producer.implementation == "clipmill-worker-audio-events@0.1.0+ced-mini"
