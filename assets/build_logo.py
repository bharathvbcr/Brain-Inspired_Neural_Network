#!/usr/bin/env python3
"""Build the BINN mark and wordmark lockups.

Source: logo-mark.svg (K=4 dendritic branches, k-WTA sparsity, somatic spike).
Run from this directory:  python3 build_logo.py
Requires macOS (qlmanage) + Pillow.
"""
import pathlib
import re
import shutil
import subprocess
from PIL import Image, ImageDraw, ImageFont

HERE = pathlib.Path(__file__).parent
SOURCE = HERE / "logo-mark.svg"
PNG = HERE / "png"
FONT = "/System/Library/Fonts/SFNSMono.ttf"
WORDMARK = "BINN"
INK_LIGHT = (0x11, 0x07, 0x0D)  # wordmark on light pages
INK_DARK = (0xFC, 0xE7, 0xF3)   # wordmark on dark pages


def render_mark(size: int = 1024) -> Image.Image:
    """Render logo-mark.svg with qlmanage and restore analytical squircle alpha."""
    tmp = HERE / "_tmp"
    shutil.rmtree(tmp, ignore_errors=True)
    tmp.mkdir()
    try:
        svg = re.sub(
            r'width="128" height="128"',
            f'width="{size}" height="{size}"',
            SOURCE.read_text(),
            count=1,
        )
        p = tmp / "mark.svg"
        p.write_text(svg)
        subprocess.run(
            ["qlmanage", "-t", "-s", str(size), "-o", str(tmp), str(p)],
            capture_output=True,
            check=True,
        )
        im = Image.open(tmp / "mark.svg.png").convert("RGBA").resize((size, size), Image.LANCZOS)
        ss = 4
        m = Image.new("L", (size * ss, size * ss), 0)
        k = size * ss / 128.0
        ImageDraw.Draw(m).rounded_rectangle(
            [6 * k, 6 * k, 122 * k, 122 * k], radius=26 * k, fill=255
        )
        im.putalpha(m.resize((size, size), Image.LANCZOS))
        return im
    finally:
        shutil.rmtree(tmp, ignore_errors=True)


def draw_tracked(d: ImageDraw.ImageDraw, xy: tuple[int, int], text: str, font, fill, tracking: float) -> float:
    """Draw text with explicit per-glyph tracking."""
    x, y = xy
    for ch in text:
        d.text((x, y), ch, font=font, fill=fill)
        x += font.getlength(ch) * (1.0 + tracking)
    return x


def lockup(mark: Image.Image, rgb: tuple[int, int, int], out: pathlib.Path, mark_px: int = 1024, tracking: float = -0.06) -> tuple[int, int]:
    """Compose square mark + wordmark lockup and crop tightly to content."""
    font = ImageFont.truetype(FONT, int(mark_px * 0.50))
    font.set_variation_by_name("Semibold")
    width = sum(font.getlength(c) * (1 + tracking) for c in WORDMARK)
    asc, _desc = font.getmetrics()
    gap, pad = int(mark_px * 0.14), int(mark_px * 0.08)
    tail_pad = int(font.getlength(WORDMARK[-1]) * 0.5)
    canvas = Image.new(
        "RGBA",
        (pad * 2 + mark_px + gap + int(width) + tail_pad, mark_px + pad * 2),
        (0, 0, 0, 0),
    )
    canvas.alpha_composite(mark.resize((mark_px, mark_px), Image.LANCZOS), (pad, pad))
    d = ImageDraw.Draw(canvas)
    draw_tracked(
        d,
        (pad + mark_px + gap, pad + (mark_px - asc) // 2 - int(mark_px * 0.03)),
        WORDMARK,
        font,
        rgb + (255,),
        tracking,
    )
    cropped = canvas.crop(canvas.getbbox())
    cropped.save(out)
    return cropped.size


def main() -> None:
    """Generate icon ladder and light/dark README wordmark lockups."""
    PNG.mkdir(exist_ok=True)
    mark = render_mark(1024)
    mark.save(PNG / "logo-mark@1024.png")
    for s in (512, 256, 128, 64, 32):
        mark.resize((s, s), Image.LANCZOS).save(PNG / f"logo-mark@{s}.png")
    print("  mark      ", mark.size, "+ ladder 512/256/128/64/32")
    print("  logo      ", lockup(mark, INK_LIGHT, PNG / "logo@2048.png"))
    print("  logo-dark ", lockup(mark, INK_DARK, PNG / "logo-dark@2048.png"))
    for n in ("logo", "logo-dark"):
        im = Image.open(PNG / f"{n}@2048.png")
        im.resize((900, int(im.height * 900 / im.width)), Image.LANCZOS).save(PNG / f"{n}@900.png")
    print("  900px lockups")


if __name__ == "__main__":
    main()
