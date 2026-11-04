"""Render the icons of the input method.

    uv run --with pillow crates/seokdok-tsf/assets/make_icons.py [FONT.ttc]

mode_seokdok.ico language-bar icon, conversion mode (釋)
mode_hangul.ico  language-bar icon, plain Hangul mode (한)
mode_latn.ico    language-bar icon, Latin mode (A)
"""

import sys
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

OUT = Path(__file__).resolve().parent
FONT = sys.argv[1] if len(sys.argv) > 1 else str(
    Path.home() / ".nix-profile/share/fonts/opentype/noto-cjk/NotoSansCJK-VF.otf.ttc"
)
SIZES = [16, 20, 24, 32, 48, 64, 256]

ICONS = {
    "mode_seokdok.ico": ("釋", (31, 78, 140)),
    "mode_hangul.ico": ("한", (40, 120, 110)),
    "mode_latn.ico": ("A", (90, 90, 96)),
}


def render(glyph: str, color: tuple[int, int, int], px: int) -> Image.Image:
    scale = 4
    s = px * scale
    img = Image.new("RGBA", (s, s), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    d.rounded_rectangle((0, 0, s - 1, s - 1), radius=s // 6, fill=color + (255,))
    font = ImageFont.truetype(FONT, int(s * 0.74), index=0)
    try:
        font.set_variation_by_name("Bold")
    except (OSError, AttributeError):
        pass  # a static font
    d.text((s / 2, s / 2), glyph, font=font, fill=(255, 255, 255, 255), anchor="mm")
    return img.resize((px, px), Image.LANCZOS)


for name, (glyph, color) in ICONS.items():
    images = [render(glyph, color, px) for px in SIZES]
    images[-1].save(OUT / name, format="ICO", sizes=[(px, px) for px in SIZES], append_images=images[:-1])
    print("wrote", OUT / name)
