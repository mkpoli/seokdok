"""Render the menu-bar icons of the input method as template images: the
glyph cut out of a rounded square, black on transparent, so macOS tints them
for light and dark menu bars. Each TIFF holds a 16 px and a 32 px (@2x)
image, as `tiffutil -cathidpicheck` would make; ImageMagick joins them.

    uv run --with pillow macos/Seokdok/Resources/make_icons.py [FONT.ttc]

seokdok.tiff  the input method and its 釋讀 mode (釋)
hangul.tiff   plain Hangul mode (한)
latin.tiff    Latin mode (A)
"""

import subprocess
import sys
import tempfile
from pathlib import Path

from PIL import Image, ImageChops, ImageDraw, ImageFont

OUT = Path(__file__).resolve().parent
FONT = sys.argv[1] if len(sys.argv) > 1 else str(
    Path.home() / ".nix-profile/share/fonts/opentype/noto-cjk/NotoSansCJK-VF.otf.ttc"
)
POINTS = 16

ICONS = {"seokdok.tiff": "釋", "hangul.tiff": "한", "latin.tiff": "A"}


def render(glyph: str, px: int) -> Image.Image:
    over = 8
    s = px * over
    square = Image.new("L", (s, s), 0)
    ImageDraw.Draw(square).rounded_rectangle((s // 16, s // 16, s - 1 - s // 16, s - 1 - s // 16), radius=s // 5, fill=255)
    cut = Image.new("L", (s, s), 0)
    font = ImageFont.truetype(FONT, int(s * 0.66), index=0)
    try:
        font.set_variation_by_name("Bold")
    except (OSError, AttributeError):
        pass  # a static font
    ImageDraw.Draw(cut).text((s / 2, s / 2), glyph, font=font, fill=255, anchor="mm")
    alpha = ImageChops.subtract(square, cut).resize((px, px), Image.LANCZOS)
    img = Image.new("RGBA", (px, px), (0, 0, 0, 0))
    img.putalpha(alpha)
    return img


with tempfile.TemporaryDirectory() as tmp:
    for name, glyph in ICONS.items():
        args = ["magick"]
        for scale in (1, 2):
            png = Path(tmp) / f"{name}@{scale}x.png"
            render(glyph, POINTS * scale).save(png)
            # 72 dpi per point: both images are 16 pt wide.
            args += ["(", str(png), "-set", "units", "PixelsPerInch", "-set", "density", str(72 * scale), ")"]
        args += ["-compress", "lzw", str(OUT / name)]
        subprocess.run(args, check=True)
        print("wrote", OUT / name)
