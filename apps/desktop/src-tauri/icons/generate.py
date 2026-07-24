#!/usr/bin/env python3
"""Regenerate the application icons.

The mark is the same scissor-blade glyph the sidebar draws, rendered onto a
rounded indigo tile. Kept as a generator rather than committed binary art of
unknown provenance: the shapes below are the whole source, and every icon in
this directory is reproducible with `python3 generate.py`.

Depends only on the standard library, so it runs in CI without adding an
image toolchain to the build.
"""

from __future__ import annotations

import struct
import zlib
from pathlib import Path

ACCENT = (0x5E, 0x6A, 0xD2)
FOREGROUND = (0xFF, 0xFF, 0xFF)
SUPERSAMPLE = 3

# Mark geometry in the 24x24 space the SVG uses.
CIRCLES = ((6.0, 6.0, 2.5), (6.0, 18.0, 2.5))
SEGMENTS = (((8.1, 7.6), (20.0, 18.0)), ((8.1, 16.4), (20.0, 6.0)))
STROKE = 1.5
CORNER_RADIUS = 4.6


def _rounded_tile(x: float, y: float) -> bool:
    """A 24x24 rounded square covering the whole canvas."""
    radius = CORNER_RADIUS
    cx = min(max(x, radius), 24.0 - radius)
    cy = min(max(y, radius), 24.0 - radius)
    return (x - cx) ** 2 + (y - cy) ** 2 <= radius**2


def _on_ring(x: float, y: float) -> bool:
    half = STROKE / 2.0
    for cx, cy, r in CIRCLES:
        distance = ((x - cx) ** 2 + (y - cy) ** 2) ** 0.5
        if r - half <= distance <= r + half:
            return True
    return False


def _on_segment(x: float, y: float) -> bool:
    half = STROKE / 2.0
    for (x0, y0), (x1, y1) in SEGMENTS:
        dx, dy = x1 - x0, y1 - y0
        length_squared = dx * dx + dy * dy
        t = ((x - x0) * dx + (y - y0) * dy) / length_squared
        t = min(max(t, 0.0), 1.0)
        px, py = x0 + t * dx, y0 + t * dy
        if ((x - px) ** 2 + (y - py) ** 2) ** 0.5 <= half:
            return True
    return False


def _pixel(size: int, px: int, py: int) -> tuple[int, int, int, int]:
    """Supersample one pixel of the mark into straight RGBA."""
    tile_hits = 0
    mark_hits = 0
    samples = SUPERSAMPLE * SUPERSAMPLE
    for sy in range(SUPERSAMPLE):
        for sx in range(SUPERSAMPLE):
            x = (px + (sx + 0.5) / SUPERSAMPLE) * 24.0 / size
            y = (py + (sy + 0.5) / SUPERSAMPLE) * 24.0 / size
            if not _rounded_tile(x, y):
                continue
            tile_hits += 1
            if _on_ring(x, y) or _on_segment(x, y):
                mark_hits += 1

    if tile_hits == 0:
        return (0, 0, 0, 0)

    coverage = mark_hits / samples
    tile = tile_hits / samples
    # Composite the white mark over the indigo tile, then premultiply by the
    # tile's own coverage so the rounded edge stays smooth.
    channels = tuple(
        round(ACCENT[i] * (1.0 - coverage) + FOREGROUND[i] * coverage) for i in range(3)
    )
    return (channels[0], channels[1], channels[2], round(255 * tile))


def _chunk(tag: bytes, payload: bytes) -> bytes:
    return (
        struct.pack(">I", len(payload))
        + tag
        + payload
        + struct.pack(">I", zlib.crc32(tag + payload) & 0xFFFFFFFF)
    )


def write_png(path: Path, size: int) -> None:
    raw = bytearray()
    for py in range(size):
        raw.append(0)  # filter type 0 (None)
        for px in range(size):
            raw.extend(_pixel(size, px, py))

    header = struct.pack(">IIBBBBB", size, size, 8, 6, 0, 0, 0)
    png = (
        b"\x89PNG\r\n\x1a\n"
        + _chunk(b"IHDR", header)
        + _chunk(b"IDAT", zlib.compress(bytes(raw), 9))
        + _chunk(b"IEND", b"")
    )
    path.write_bytes(png)
    print(f"icons: wrote {path.name} ({size}x{size})")


def main() -> None:
    here = Path(__file__).parent
    write_png(here / "32x32.png", 32)
    write_png(here / "128x128.png", 128)
    write_png(here / "128x128@2x.png", 256)
    write_png(here / "icon.png", 512)


if __name__ == "__main__":
    main()
