# Asset Sources

This folder stores source-of-truth files and helper scripts for regenerating derived UI assets.

## Files

- `banner-original.png`: Original pre-icon banner source.
- `icon.ico`: Source icon used for regenerating the banner artwork.
- `regenerate_banner.py`: Rebuilds `src/assets/banner.png` by overlaying `assets-source/icon.ico`.

## Regenerate `banner.png`

From repo root:

```bash
nix-shell -p python3 python3Packages.pillow --run "python3 assets-source/regenerate_banner.py --x 33 --y 16"
```

Notes:

- `--x` and `--y` set the icon position.
- `--scale` controls icon height relative to the banner text area (default `0.82`).
- Output is always written to `src/assets/banner.png`.
