"""Generate the Lumo icon set from the supplied source artwork."""

from PIL import Image
import os

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SOURCE = os.path.join(ROOT, "assets", "lumo-icon-source.png")


def save(img, path):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    img.save(path, "PNG")
    print("wrote", path)


def main():
    src = Image.open(SOURCE).convert("RGBA")
    # Normalise to a square master; all sizes derive from it for consistent edges.
    master = src.resize((1024, 1024), Image.LANCZOS) if src.size != (1024, 1024) else src
    os.makedirs(os.path.join(ROOT, "icons"), exist_ok=True)
    save(master.resize((512, 512), Image.LANCZOS), os.path.join(ROOT, "icons", "icon.png"))
    for name, px in (("32x32.png", 32), ("128x128.png", 128), ("128x128@2x.png", 256)):
        save(master.resize((px, px), Image.LANCZOS), os.path.join(ROOT, "icons", name))
    ico = os.path.join(ROOT, "icons", "icon.ico")
    master.resize((256, 256), Image.LANCZOS).save(
        ico,
        sizes=[
            (16, 16),
            (20, 20),
            (24, 24),
            (32, 32),
            (40, 40),
            (48, 48),
            (64, 64),
            (128, 128),
            (256, 256),
        ],
    )
    print("wrote", ico)

    ui_icons = os.path.join(ROOT, "ui", "icons")
    save(master.resize((192, 192), Image.LANCZOS), os.path.join(ui_icons, "icon-192.png"))
    save(master.resize((512, 512), Image.LANCZOS), os.path.join(ui_icons, "icon-512.png"))
    save(master.resize((180, 180), Image.LANCZOS), os.path.join(ui_icons, "apple-touch-icon.png"))
    save(master.resize((32, 32), Image.LANCZOS), os.path.join(ui_icons, "favicon.png"))
    save(master.resize((512, 512), Image.LANCZOS), os.path.join(ui_icons, "icon-maskable-512.png"))


if __name__ == "__main__":
    main()

