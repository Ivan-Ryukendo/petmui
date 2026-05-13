#!/usr/bin/env python3
"""Convert hatch-pet/Codex pet output into petmui's tiny runtime package format.

Input can be a hatch-pet or Codex pet package directory containing pet.json
plus spritesheet.png/webp, or a direct path to a spritesheet image.

Requires Pillow:
    python -m pip install pillow
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path

try:
    from PIL import Image, ImageDraw, ImageFont
except ImportError as exc:
    raise SystemExit("Pillow is required: python -m pip install pillow") from exc


DEFAULT_ROWS = [
    "idle",
    "waving",
    "jumping",
    "failed",
    "review",
    "running-right",
    "running-left",
    "running",
    "working",
]


def find_spritesheet(source: Path) -> Path:
    if source.is_file():
        return source
    manifest = read_json(source / "pet.json")
    if manifest:
        declared = manifest.get("spritesheetPath") or manifest.get("spritesheet") or manifest.get("atlas")
        if isinstance(declared, str):
            candidate = source / declared
            if candidate.exists():
                return candidate
    for name in ("spritesheet.png", "spritesheet.webp", "final/spritesheet.png", "final/spritesheet.webp"):
        candidate = source / name
        if candidate.exists():
            return candidate
    raise SystemExit(f"No spritesheet found in {source}")


def read_source_name(source: Path) -> str:
    data = read_json(source / "pet.json")
    if data:
        return str(data.get("displayName") or data.get("name") or data.get("id") or source.name)
    return source.stem if source.is_file() else source.name


def read_json(path: Path) -> dict:
    if not path.exists():
        return {}
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError:
        return {}
    return data if isinstance(data, dict) else {}


def infer_rows(image_size: tuple[int, int], cell_width: int, cell_height: int, requested_rows: str) -> list[str]:
    rows = [row.strip() for row in requested_rows.split(",") if row.strip()]
    expected_height = cell_height * len(rows)
    if image_size[1] == expected_height:
        return rows
    actual_rows = image_size[1] // cell_height
    if actual_rows <= 0 or image_size[1] % cell_height != 0:
        raise SystemExit(f"Image height {image_size[1]} is not divisible by cell height {cell_height}")
    if actual_rows <= len(DEFAULT_ROWS):
        return DEFAULT_ROWS[:actual_rows]
    return DEFAULT_ROWS + [f"extra-{index + 1}" for index in range(actual_rows - len(DEFAULT_ROWS))]


def write_config(config_path: Path, pet_dir: Path) -> None:
    relative = pet_dir
    try:
        relative = pet_dir.relative_to(config_path.parent)
    except ValueError:
        pass
    config_path.write_text(
        "\n".join(
            [
                "pet_size = 96",
                f'pet_directory = "{relative.as_posix()}"',
                "enable_typing_detection = true",
                "click_through_in_games = true",
                "",
            ]
        ),
        encoding="utf-8",
    )


def fit_static_image(image: Image.Image, size: int) -> Image.Image:
    image = image.convert("RGBA")
    scale = min(size / image.width, size / image.height)
    width = max(1, round(image.width * scale))
    height = max(1, round(image.height * scale))
    resampling = getattr(Image, "Resampling", Image).LANCZOS
    image = image.resize((width, height), resampling)
    canvas = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    canvas.alpha_composite(image, ((size - width) // 2, (size - height) // 2))
    return canvas


def default_emoji_font(size: int, requested: str | None) -> ImageFont.FreeTypeFont | ImageFont.ImageFont:
    candidates = []
    if requested:
        candidates.append(Path(requested))
    candidates.extend(
        [
            Path("C:/Windows/Fonts/seguiemj.ttf"),
            Path("C:/Windows/Fonts/seguisym.ttf"),
            Path("C:/Windows/Fonts/arial.ttf"),
        ]
    )
    for path in candidates:
        if path.exists():
            try:
                return ImageFont.truetype(str(path), size)
            except OSError:
                pass
    return ImageFont.load_default()


def render_text_image(text: str, size: int, font_path: str | None) -> Image.Image:
    image = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(image)
    font = default_emoji_font(max(12, int(size * 0.72)), font_path)
    try:
        bbox = draw.textbbox((0, 0), text, font=font, embedded_color=True)
    except TypeError:
        bbox = draw.textbbox((0, 0), text, font=font)
    x = (size - (bbox[2] - bbox[0])) // 2 - bbox[0]
    y = (size - (bbox[3] - bbox[1])) // 2 - bbox[1]
    try:
        draw.text((x, y), text, font=font, embedded_color=True, fill=(255, 255, 255, 255))
    except TypeError:
        draw.text((x, y), text, font=font, fill=(255, 255, 255, 255))
    return image


def write_static_package(output: Path, image: Image.Image, name: str, write_config_path: Path | None) -> None:
    output.mkdir(parents=True, exist_ok=True)
    (output / "image.bgra").write_bytes(image.tobytes("raw", "BGRA"))
    manifest = {
        "name": name,
        "renderer": "static-bgra-v1",
        "image": "image.bgra",
        "width": image.width,
        "height": image.height,
    }
    (output / "pet.json").write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    if write_config_path:
        write_config(write_config_path.resolve(), output)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("source", type=Path, nargs="?", help="hatch-pet package directory, spritesheet image, or static output when --emoji is used")
    parser.add_argument("output", type=Path, nargs="?", help="output pet package directory")
    parser.add_argument("--name", help="display name for the converted pet")
    parser.add_argument(
        "--static",
        action="store_true",
        help="import source as a single static image instead of a spritesheet",
    )
    parser.add_argument(
        "--emoji",
        help="render emoji/text into a static pet package; Pillow/font support controls color emoji quality",
    )
    parser.add_argument("--static-size", type=int, default=192, help="square pixel size for static image packages")
    parser.add_argument("--font", help="optional TrueType/OpenType font path for --emoji rendering")
    parser.add_argument("--cell-width", type=int, default=192)
    parser.add_argument("--cell-height", type=int, default=208)
    parser.add_argument("--columns", type=int, default=8)
    parser.add_argument(
        "--rows",
        default=",".join(DEFAULT_ROWS),
        help="comma-separated row names in top-to-bottom atlas order",
    )
    parser.add_argument(
        "--write-config",
        type=Path,
        help="optional config.toml path to update with pet_directory",
    )
    args = parser.parse_args()

    if args.static_size <= 0:
        raise SystemExit("--static-size must be greater than zero")

    if args.emoji:
        if args.output is None and args.source is not None:
            output = args.source.resolve()
        elif args.output is not None:
            output = args.output.resolve()
        else:
            raise SystemExit("Output directory is required when using --emoji")
        image = render_text_image(args.emoji, args.static_size, args.font)
        write_static_package(output, image, args.name or args.emoji, args.write_config)
        print(f"Wrote {output}")
        return

    if args.source is None or args.output is None:
        raise SystemExit("Source and output are required")

    source = args.source.resolve()
    output = args.output.resolve()

    if args.static:
        image = fit_static_image(Image.open(source), args.static_size)
        write_static_package(output, image, args.name or read_source_name(source), args.write_config)
        print(f"Wrote {output}")
        return

    spritesheet = find_spritesheet(source)

    image = Image.open(spritesheet).convert("RGBA")
    rows = infer_rows(image.size, args.cell_width, args.cell_height, args.rows)
    if not rows:
        raise SystemExit("At least one row name is required")
    expected = (args.cell_width * args.columns, args.cell_height * len(rows))
    if image.size != expected:
        raise SystemExit(f"Expected spritesheet size {expected}, got {image.size}")

    output.mkdir(parents=True, exist_ok=True)
    (output / "atlas.bgra").write_bytes(image.tobytes("raw", "BGRA"))

    manifest = {
        "name": args.name or read_source_name(source),
        "renderer": "raw-bgra-atlas-v1",
        "atlas": "atlas.bgra",
        "cellWidth": args.cell_width,
        "cellHeight": args.cell_height,
        "columns": args.columns,
        "rows": rows,
    }
    (output / "pet.json").write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    if args.write_config:
        write_config(args.write_config.resolve(), output)
    print(f"Wrote {output}")


if __name__ == "__main__":
    main()
