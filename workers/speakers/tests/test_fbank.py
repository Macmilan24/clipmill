"""The filterbank, value for value against torchaudio's Kaldi implementation.

The voice-print model was trained on torchaudio's features; features that
merely look similar would give prints that merely look similar. The reference
was computed once by `torchaudio.compliance.kaldi.fbank` over the signal
below and is pinned in `data/fbank_reference.json`.
"""

from __future__ import annotations

import json
from pathlib import Path

import numpy as np
from clipmill_worker_speakers.fbank import MEL_BINS, fbank, frame_count

REFERENCE = json.loads((Path(__file__).parent / "data" / "fbank_reference.json").read_text())


def signal() -> np.ndarray:
    """Half a second: two tones, a sweep and seeded noise, at int16 scale."""

    rate = 16_000
    t = np.arange(rate // 2) / rate
    rng = np.random.default_rng(20260927)
    wave = (
        3000 * np.sin(2 * np.pi * 220 * t)
        + 1500 * np.sin(2 * np.pi * 1760 * t)
        + 800 * np.sin(2 * np.pi * (300 + 4000 * t) * t)
        + rng.normal(0, 200, t.shape)
    )
    return np.round(wave).astype(np.int16).astype(np.float32)


def test_the_features_are_torchaudios_to_a_thousandth() -> None:
    features = fbank(signal())
    assert features.shape == (REFERENCE["frame_count"], MEL_BINS)
    assert features.dtype == np.float32
    for index, expected in REFERENCE["frames"].items():
        difference = np.abs(features[int(index)] - np.array(expected, np.float32))
        assert difference.max() < 2e-3, f"frame {index} is off by {difference.max()}"


def test_frames_are_counted_as_kaldi_counts_them() -> None:
    assert frame_count(399) == 0
    assert frame_count(400) == 1
    assert frame_count(559) == 1
    assert frame_count(560) == 2
    assert frame_count(24_000) == 148
    assert fbank(np.zeros(100, np.float32)).shape == (0, MEL_BINS)


def test_silence_is_floored_rather_than_minus_infinity() -> None:
    features = fbank(np.zeros(4_000, np.float32))
    assert np.isfinite(features).all()
