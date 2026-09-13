#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
# Copyright 2026 kodephp contributors

"""Generate the ntfs-mac application icon and menu-bar tray glyph.

Why a script instead of hand-made assets: the previous icon was a 128x128
solid blue square (359 bytes), which made the app look like a placeholder in
the Dock and in the Finder. This script renders the icon set from source so
it is reproducible and diffable.

Outputs (under `crates/ntfs-mac-tauri/src-tauri/icons/`):
  - source/icon-1024.png   brand icon, fed into `cargo tauri icon`
  - tray/tray-22.png       macOS menu-bar *template* image (black + alpha)
  - tray/tray-44.png       same glyph at 2x

`cargo tauri icon crates/ntfs-mac-tauri/src-tauri/icons/source/icon-1024.png`
derives the rest of the set (icon.png, icon@2x.png, icon_32x32.png, …,
icon.icns, StoreLogo.png).

Requires Pillow: `python3 -m pip install pillow`
"""

from __future__ import annotations

import sys
from pathlib import Path

try:
    from PIL import Image, ImageDraw, ImageFont
except ImportError:  # pragma: no cover
    sys.exit("pillow is required: python3 -m pip install pillow")

ROOT = Path(__file__).resolve().parent.parent
ICON_DIR = ROOT / "crates" / "ntfs-mac-tauri" / "src-tauri" / "icons"
BRAND = ICON_DIR / "source"
TRAY = ICON_DIR / "tray"

# Brand palette: blue in the Apple system range, ntfs-mac's own shade.
COLOR_TOP = (96, 168, 255, 255)
COLOR_BOTTOM = (25, 106, 224, 255)
COLOR_WHITE = (255, 255, 255, 255)

# macOS system fonts, first hit wins.
FONT_CANDIDATES = [
    "/System/Library/Fonts/SFNSDisplay.ttf",
    "/System/Library/Fonts/Helvetica.ttc",
    "/System/Library/Fonts/Supplemental/Arial Bold.ttf",
    "/System/Library/Fonts/Supplemental/Arial.ttf",
]


def _pick_font(size: int) -> ImageFont.FreeTypeFont | ImageFont.ImageFont:
    for candidate in FONT_CANDIDATES:
        if Path(candidate).exists():
            try:
                return ImageFont.truetype(candidate, size)
            except OSError:  # pragma: no cover
                continue
    return ImageFont.load_default()


def _rounded_tile(size: int, inset: float, radius: float) -> Image.Image:
    """Vertical gradient clipped to a rounded tile."""
    gradient = Image.new("RGBA", (1, size))
    gd = ImageDraw.Draw(gradient)
    for y in range(size):
        t = y / max(1, size - 1)
        gd.point((0, y), tuple(int(a + (b - a) * t) for a, b in zip(COLOR_TOP, COLOR_BOTTOM)))
    gradient = gradient.resize((size, size))

    mask = Image.new("L", (size, size), 0)
    i, r = int(inset * size), int(radius * size)
    ImageDraw.Draw(mask).rounded_rectangle([i, i, size - i - 1, size - i - 1], radius=r, fill=255)
    return Image.composite(gradient, Image.new("RGBA", (size, size), (0, 0, 0, 0)), mask)


def brand_icon(size: int = 1024) -> Image.Image:
    """Gradient tile + hard-drive glyph + NTFS wordmark."""
    img = _rounded_tile(size, 0.0625, 0.222)
    tile = Image.new("RGBA", (size, size), (0, 0, 0, 0))

    # Subtle top highlight for depth.
    hl = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    i, r = int(0.0625 * size), int(0.222 * size)
    ImageDraw.Draw(hl).rounded_rectangle([i, i, size - i, int(0.52 * size)], radius=r, fill=(255, 255, 255, 42))
    tile.alpha_composite(hl)

    draw = ImageDraw.Draw(tile)

    # Drive body.
    body_top, body_bottom = int(0.36 * size), int(0.52 * size)
    body_left, body_right = int(0.235 * size), int(0.765 * size)
    draw.rounded_rectangle(
        [body_left, body_top, body_right, body_bottom],
        radius=int(0.045 * size),
        fill=COLOR_WHITE,
    )

    # Details cut back into the gradient colour so they read as *inside* the
    # drive, not as separate shapes floating on the tile.
    cut = COLOR_BOTTOM
    mid = (body_top + body_bottom) // 2

    led_r = int(0.026 * size)
    led_cx = int(0.295 * size)
    draw.ellipse([led_cx - led_r, mid - led_r, led_cx + led_r, mid + led_r], fill=cut)

    bar_h = int(0.017 * size)
    draw.rounded_rectangle(
        [int(0.365 * size), mid - bar_h // 2, int(0.625 * size), mid - bar_h // 2 + bar_h],
        radius=bar_h // 2,
        fill=cut,
    )
    draw.rounded_rectangle(
        [int(0.365 * size), mid + int(0.024 * size), int(0.525 * size), mid + int(0.024 * size) + bar_h],
        radius=bar_h // 2,
        fill=cut,
    )

    # Wordmark.
    font = _pick_font(int(0.175 * size))
    bbox = draw.textbbox((0, 0), "NTFS", font=font)
    draw.text(
        ((size - (bbox[2] - bbox[0])) / 2 - bbox[0], int(0.635 * size) - bbox[1]),
        "NTFS",
        font=font,
        fill=COLOR_WHITE,
    )

    img.alpha_composite(tile)
    return img


def tray_template(size: int) -> Image.Image:
    """Menu-bar glyph: hard-drive outline with an LED.

    macOS template images are pure black with alpha so the system tints them
    correctly in both light and dark menu bars. Geometry is defined in a 44px
    space and scaled, so both 1x and 2x variants stay centred.
    """
    alpha = Image.new("L", (size, size), 0)
    d = ImageDraw.Draw(alpha)
    k = size / 44.0
    lw = max(1, round(3 * k))

    d.rounded_rectangle(
        [7 * k, 14 * k, 37 * k, 34 * k],
        radius=5 * k,
        outline=255,
        width=lw,
    )
    cx, cy, r = 15 * k, 24 * k, 2.6 * k
    d.ellipse([cx - r, cy - r, cx + r, cy + r], fill=255)
    d.rectangle([23 * k, 23 * k, 32 * k, 25 * k], fill=255)

    rgb = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    return Image.composite(Image.new("RGBA", (size, size), (0, 0, 0, 255)), rgb, alpha)


def main() -> int:
    BRAND.mkdir(parents=True, exist_ok=True)
    TRAY.mkdir(parents=True, exist_ok=True)

    src = BRAND / "icon-1024.png"
    brand_icon(1024).save(src, "PNG")
    print(f"✓ {src.relative_to(ROOT)}  ({src.stat().st_size:,} bytes)")

    for name, size in (("tray-22.png", 22), ("tray-44.png", 44)):
        out = TRAY / name
        tray_template(size).save(out, "PNG")
        print(f"✓ {out.relative_to(ROOT)}  ({out.stat().st_size:,} bytes)")

    print("\n下一步：cargo tauri icon crates/ntfs-mac-tauri/src-tauri/icons/source/icon-1024.png")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
