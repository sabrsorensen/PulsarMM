#!/usr/bin/env python3
"""Regenerate src/assets/banner.png from source assets.

Usage:
  python3 assets-source/regenerate_banner.py --x 33 --y 16
"""

from __future__ import annotations

import argparse
from pathlib import Path

from PIL import Image


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Regenerate banner.png with icon overlay.")
    parser.add_argument("--x", type=int, default=33, help="Icon X position (default: 33)")
    parser.add_argument("--y", type=int, default=16, help="Icon Y position (default: 16)")
    parser.add_argument(
        "--scale",
        type=float,
        default=0.82,
        help="Icon height scale relative to banner text content height (default: 0.82)",
    )
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    repo_root = Path(__file__).resolve().parent.parent
    source_banner = repo_root / "assets-source" / "banner-original.png"
    icon_file = repo_root / "assets-source" / "icon.ico"
    out_banner = repo_root / "src" / "assets" / "banner.png"

    if not source_banner.exists():
        raise FileNotFoundError(f"Missing source banner: {source_banner}")
    if not icon_file.exists():
        raise FileNotFoundError(f"Missing icon file: {icon_file}")

    base = Image.open(source_banner).convert("RGBA")
    icon = Image.open(icon_file).convert("RGBA")

    # Trim transparent icon margins for precise positioning.
    icon_bbox = icon.split()[-1].getbbox()
    if icon_bbox:
        icon = icon.crop(icon_bbox)

    visible_bbox = base.split()[-1].getbbox()
    if not visible_bbox:
        raise RuntimeError("Source banner has no visible pixels.")

    _, top, _, bottom = visible_bbox
    content_h = bottom - top
    icon_h = max(1, int(content_h * args.scale))
    icon_w = max(1, int(icon.width * (icon_h / icon.height)))
    icon = icon.resize((icon_w, icon_h), Image.Resampling.LANCZOS)

    x = max(0, min(base.width - icon_w, args.x))
    y = max(0, min(base.height - icon_h, args.y))

    out = base.copy()
    out.alpha_composite(icon, (x, y))
    out.save(out_banner)

    print(f"Saved {out_banner}")
    print(f"Icon size: {icon_w}x{icon_h}")
    print(f"Icon position: x={x}, y={y}")


if __name__ == "__main__":
    main()
