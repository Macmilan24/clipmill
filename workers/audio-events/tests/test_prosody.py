from types import SimpleNamespace

import numpy as np
import pytest
from clipmill_worker_audio_events.prosody import RATE, emphasis, speech_ranges


def tone(seconds, hz, amplitude):
    time = np.arange(round(seconds * RATE)) / RATE
    return (amplitude * np.sin(2 * np.pi * hz * time)).astype(np.float32)


@pytest.mark.parametrize(
    ("hz", "amplitude", "pitch_change", "level_change"),
    [(200, 0.08, 6.18, 0), (140, 0.20, 0, 7.96), (180, 0.12, 4.35, 3.52)],
)
def test_a_sustained_accent_reports_measured_pitch_and_level(
    hz, amplitude, pitch_change, level_change
):
    audio = np.concatenate([tone(2, 140, 0.08), tone(0.5, hz, amplitude), tone(1, 140, 0.08)])
    regions = [(0, len(audio))]
    (event,) = emphasis(audio, regions, lambda: None)
    assert event.start_sample / RATE == pytest.approx(2, abs=0.05)
    assert event.end_sample / RATE == pytest.approx(2.5, abs=0.05)
    assert event.delivery.pitch_hz == pytest.approx(hz, abs=3)
    assert event.delivery.pitch_change_semitones == pytest.approx(pitch_change, abs=0.4)
    assert event.delivery.level_change_db == pytest.approx(level_change, abs=0.5)
    assert 0.72 <= event.mean <= event.peak <= 1
    assert emphasis(audio, regions, lambda: None) == [event]


def test_silence_noise_clicks_and_uncovered_tones_are_not_speech_accents():
    rng = np.random.default_rng(91)
    speech = tone(2, 140, 0.06)
    noisy = rng.normal(0, 0.22, RATE).astype(np.float32)
    silence = np.zeros(RATE, dtype=np.float32)
    silence[RATE // 2] = 0.9
    audio = np.concatenate([speech, noisy, silence, tone(0.5, 250, 0.24)])
    assert emphasis(audio, [(0, 4 * RATE)], lambda: None) == []
    assert emphasis(audio, [], lambda: None) == []
    assert (
        emphasis(np.concatenate([speech, tone(0.5, 250, 0.24)]), [(0, len(speech))], lambda: None)
        == []
    )
    assert emphasis(tone(0.2, 250, 0.4), [(0, RATE)], lambda: None) == []


def test_speech_ranges_exclude_invalid_or_unmeasured_samples():
    def span(a, b):
        return SimpleNamespace(start_ticks=a, end_ticks=b)

    vad = SimpleNamespace(
        segments=[span(0, 180_000), span(270_000, 360_000)],
        coverage=SimpleNamespace(start_ticks=45_000, end_ticks=315_000, analyzed=True),
        invalid_regions=[span(135_000, 280_000), span(70_000, 90_000)],
    )
    assert list(speech_ranges(vad)) == [(8_000, 12_444), (16_000, 24_000), (49_778, 56_000)]
    vad.coverage.analyzed = False
    assert not list(speech_ranges(vad))
