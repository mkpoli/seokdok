"""Draw the app icon on every platform: white 釋 on the site's indigo.

Usage: uv run --no-project --with pillow --with fonttools tools/make_app_icon.py

The glyph is an outline from Noto Serif CJK KR Black (SIL OFL 1.1), so no device
needs the font. Writes web/public/favicon.svg, the iOS and macOS asset catalogs,
the Android adaptive icon and the Windows crates/seokdok-tsf/assets/icon.ico.
"""

import json
from pathlib import Path

from fontTools.pens.svgPathPen import SVGPathPen
from fontTools.pens.transformPen import TransformPen
from fontTools.pens.boundsPen import BoundsPen
from fontTools.ttLib import TTCollection
from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parent.parent
FONT = "/usr/share/fonts/opentype/noto/NotoSerifCJK-Black.ttc"
INDIGO = "#24348c"
WHITE = "#ffffff"
# Share of the icon's width taken by the glyph on a full-bleed square.
GLYPH_SHARE = 0.72


def font():
    for f in TTCollection(FONT).fonts:
        if "Noto Serif CJK KR" in f["name"].getDebugName(1):
            return f
    raise SystemExit("Noto Serif CJK KR not found")


def outline(size: float, share: float, y_down: bool = True, pad: float = 0.0) -> str:
    """SVG path of 釋, centred by its ink in a size-wide square."""
    f = font()
    glyphs = f.getGlyphSet()
    glyph = glyphs[f.getBestCmap()[ord("釋")]]
    box = BoundsPen(glyphs)
    glyph.draw(box)
    x0, y0, x1, y1 = box.bounds
    scale = size * share / max(x1 - x0, y1 - y0)
    dx = pad + size / 2 - scale * (x0 + x1) / 2
    dy = pad + size / 2 + scale * (y0 + y1) / 2
    pen = SVGPathPen(glyphs, ntos=lambda v: f"{v:.2f}".rstrip("0").rstrip("."))
    glyph.draw(TransformPen(pen, (scale, 0, 0, -scale, dx, dy)))
    return pen.getCommands()


def raster(size: int, radius: float = 0.0, inset: float = 0.0, share: float = GLYPH_SHARE, alpha: bool = False) -> Image.Image:
    """Render through an oversampled mask; `inset` shrinks the tile within the canvas."""
    k = 8
    big = size * k
    f = font()
    glyphs = f.getGlyphSet()
    glyph = glyphs[f.getBestCmap()[ord("釋")]]
    box = BoundsPen(glyphs)
    glyph.draw(box)
    x0, y0, x1, y1 = box.bounds
    tile = big * (1 - 2 * inset)
    scale = tile * share / max(x1 - x0, y1 - y0)
    dx = big / 2 - scale * (x0 + x1) / 2
    dy = big / 2 + scale * (y0 + y1) / 2
    canvas = Image.new("RGBA" if alpha else "RGB", (big, big), (0, 0, 0, 0) if alpha else INDIGO)
    d = ImageDraw.Draw(canvas)
    if alpha:
        m = big * inset
        d.rounded_rectangle((m, m, big - m, big - m), radius=tile * radius, fill=INDIGO)
    mask = Image.new("L", (big, big), 0)
    contours = flatten(glyph, glyphs, scale, dx, dy)
    draw_evenodd(mask, contours)
    canvas.paste(Image.new(canvas.mode, (big, big), WHITE), (0, 0), mask)
    return canvas.resize((size, size), Image.LANCZOS)


def flatten(glyph, glyphs, scale, dx, dy):
    from fontTools.pens.basePen import BasePen

    class Flat(BasePen):
        def __init__(self):
            super().__init__(glyphs)
            self.contours = []

        def _moveTo(self, p):
            self.contours.append([p])

        def _lineTo(self, p):
            self.contours[-1].append(p)

        def _curveToOne(self, a, b, c):
            p0 = self.contours[-1][-1]
            for i in range(1, 17):
                t = i / 16
                u = 1 - t
                self.contours[-1].append(tuple(u**3 * p0[j] + 3 * u * u * t * a[j] + 3 * u * t * t * b[j] + t**3 * c[j] for j in (0, 1)))

    pen = Flat()
    glyph.draw(TransformPen(pen, (scale, 0, 0, -scale, dx, dy)))
    return pen.contours


def draw_evenodd(mask: Image.Image, contours) -> None:
    """Non-zero fill: outer contours fill, counters (opposite winding) clear."""
    def area(c):
        return sum(c[i][0] * c[(i + 1) % len(c)][1] - c[(i + 1) % len(c)][0] * c[i][1] for i in range(len(c))) / 2

    d = ImageDraw.Draw(mask)
    ordered = sorted(contours, key=lambda c: -abs(area(c)))
    sign = area(ordered[0])
    for c in ordered:
        d.polygon(c, fill=255 if area(c) * sign > 0 else 0)


def write_json(p: Path, data) -> None:
    p.parent.mkdir(parents=True, exist_ok=True)
    p.write_text(json.dumps(data, indent=2) + "\n", encoding="utf-8")


def favicon() -> None:
    path = outline(64, 0.68)
    (ROOT / "web/public/favicon.svg").write_text(
        f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64"><rect width="64" height="64" rx="14" fill="{INDIGO}"/><path fill="{WHITE}" d="{path}"/></svg>\n',
        encoding="utf-8",
    )


def ios() -> None:
    d = ROOT / "ios/App/Assets.xcassets"
    write_json(d / "Contents.json", {"info": {"author": "xcode", "version": 1}})
    s = d / "AppIcon.appiconset"
    s.mkdir(parents=True, exist_ok=True)
    # App Store icons are square, opaque and full-bleed; iOS rounds them.
    raster(1024).convert("RGB").save(s / "icon-1024.png")
    write_json(s / "Contents.json", {
        "images": [{"filename": "icon-1024.png", "idiom": "universal", "platform": "ios", "size": "1024x1024"}],
        "info": {"author": "xcode", "version": 1},
    })


def macos() -> None:
    d = ROOT / "macos/Seokdok/Assets.xcassets"
    write_json(d / "Contents.json", {"info": {"author": "xcode", "version": 1}})
    s = d / "AppIcon.appiconset"
    s.mkdir(parents=True, exist_ok=True)
    images = []
    # Apple's macOS grid: an 824-point rounded tile on a 1024-point canvas.
    for pt in (16, 32, 128, 256, 512):
        for mult in (1, 2):
            px = pt * mult
            name = f"icon-{px}.png"
            if not (s / name).exists():
                raster(px, radius=0.2237, inset=100 / 1024, alpha=True).save(s / name)
            images.append({"filename": name, "idiom": "mac", "scale": f"{mult}x", "size": f"{pt}x{pt}"})
    write_json(s / "Contents.json", {"images": images, "info": {"author": "xcode", "version": 1}})


def android() -> None:
    res = ROOT / "android/app/src/main/res"
    # Adaptive icons draw 108dp and may crop to the middle 66dp.
    path = outline(108, 0.46)
    (res / "drawable/ic_launcher_foreground.xml").write_text(
        '<?xml version="1.0" encoding="utf-8"?>\n'
        '<!-- 釋 from Noto Serif CJK KR Black (SIL OFL 1.1); drawn by tools/make_app_icon.py. -->\n'
        '<vector xmlns:android="http://schemas.android.com/apk/res/android"\n'
        '    android:width="108dp"\n    android:height="108dp"\n'
        '    android:viewportWidth="108"\n    android:viewportHeight="108">\n'
        f'    <path\n        android:fillColor="#FFFFFFFF"\n        android:pathData="{path}" />\n</vector>\n',
        encoding="utf-8",
    )
    (res / "drawable/ic_launcher.xml").unlink(missing_ok=True)
    a = res / "mipmap-anydpi-v26"
    a.mkdir(parents=True, exist_ok=True)
    (a / "ic_launcher.xml").write_text(
        '<?xml version="1.0" encoding="utf-8"?>\n'
        '<adaptive-icon xmlns:android="http://schemas.android.com/apk/res/android">\n'
        '    <background android:drawable="@color/ic_launcher_background" />\n'
        '    <foreground android:drawable="@drawable/ic_launcher_foreground" />\n'
        '    <monochrome android:drawable="@drawable/ic_launcher_foreground" />\n'
        '</adaptive-icon>\n',
        encoding="utf-8",
    )
    colors = res / "values/ic_launcher_colors.xml"
    colors.write_text(
        f'<?xml version="1.0" encoding="utf-8"?>\n<resources>\n    <color name="ic_launcher_background">#FF24348C</color>\n</resources>\n',
        encoding="utf-8",
    )


def windows() -> None:
    sizes = (16, 20, 24, 32, 40, 48, 64, 128, 256)
    frames = [raster(s, radius=0.18, alpha=True, share=0.82 if s <= 32 else GLYPH_SHARE) for s in sizes]
    frames[-1].save(ROOT / "crates/seokdok-tsf/assets/icon.ico", format="ICO", append_images=frames[:-1], sizes=[(s, s) for s in sizes])


def main() -> None:
    favicon()
    ios()
    macos()
    android()
    windows()


if __name__ == "__main__":
    main()
