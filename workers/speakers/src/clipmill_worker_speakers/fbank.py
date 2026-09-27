"""The log mel filterbank the voice-print model was trained on.

Kaldi's, as torchaudio computes it: 25 ms frames every 10 ms, the DC offset
taken out of each, a 0.97 pre-emphasis, a Hamming window, a 512-point power
spectrum, 80 triangular mel bins from 20 Hz to the Nyquist frequency, and the
natural log floored at float32's epsilon. No dither, so the same samples give
the same features. Samples are at int16 scale, as the PCM is read.

The arithmetic is float32 where torchaudio's is, so the two agree to about a
thousandth of a log unit (tests/test_fbank.py).
"""

from __future__ import annotations

import numpy as np

SAMPLE_RATE = 16_000
WINDOW = 400
SHIFT = 160
PADDED = 512
MEL_BINS = 80
PREEMPHASIS = np.float32(0.97)


def _mel(frequency: np.ndarray | float) -> np.ndarray:
    return 1127.0 * np.log(1.0 + np.asarray(frequency) / 700.0)


def _mel_banks() -> np.ndarray:
    """Triangular filters over the FFT bins, as `kaldi.get_mel_banks` builds them."""

    width = SAMPLE_RATE / PADDED
    low, high = _mel(20.0), _mel(0.5 * SAMPLE_RATE)
    delta = (high - low) / (MEL_BINS + 1)
    index = np.arange(MEL_BINS, dtype=np.float32)[:, None]
    left = np.float32(low) + index * np.float32(delta)
    centre = np.float32(low) + (index + 1) * np.float32(delta)
    right = np.float32(low) + (index + 2) * np.float32(delta)
    mel = _mel(np.float32(width) * np.arange(PADDED // 2, dtype=np.float32)).astype(np.float32)
    rising = (mel[None, :] - left) / (centre - left)
    falling = (right - mel[None, :]) / (right - centre)
    banks = np.maximum(np.float32(0), np.minimum(rising, falling))
    # No filter reaches the Nyquist bin.
    return np.pad(banks, ((0, 0), (0, 1))).astype(np.float32)


BANKS = _mel_banks()
HAMMING = (0.54 - 0.46 * np.cos(2 * np.pi * np.arange(WINDOW) / (WINDOW - 1))).astype(np.float32)
EPSILON = np.finfo(np.float32).eps


def frame_count(samples: int) -> int:
    """How many whole frames fit: Kaldi's snip-edges rule."""

    return 0 if samples < WINDOW else 1 + (samples - WINDOW) // SHIFT


def fbank(samples: np.ndarray) -> np.ndarray:
    """Frames by 80 log mel energies, float32."""

    count = frame_count(len(samples))
    if count == 0:
        return np.zeros((0, MEL_BINS), np.float32)
    index = np.arange(WINDOW)[None, :] + SHIFT * np.arange(count)[:, None]
    frames = samples[index].astype(np.float32)
    frames -= frames.mean(axis=1, keepdims=True)
    previous = np.concatenate([frames[:, :1], frames[:, :-1]], axis=1)
    frames = (frames - PREEMPHASIS * previous) * HAMMING
    frames = np.pad(frames, ((0, 0), (0, PADDED - WINDOW)))
    power = (np.abs(np.fft.rfft(frames, axis=1)) ** 2).astype(np.float32)
    return np.log(np.maximum(power @ BANKS.T, EPSILON)).astype(np.float32)


__all__ = ["MEL_BINS", "SAMPLE_RATE", "SHIFT", "WINDOW", "fbank", "frame_count"]
