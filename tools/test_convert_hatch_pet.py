#!/usr/bin/env python3
"""Smoke tests for petmui's local pet converter."""

from __future__ import annotations

import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

try:
    from PIL import Image
except ImportError:  # pragma: no cover - local convenience
    Image = None


ROOT = Path(__file__).resolve().parents[1]
CONVERTER = ROOT / "tools" / "convert_hatch_pet.py"


@unittest.skipIf(Image is None, "Pillow is required for converter smoke tests")
class ConverterSmokeTests(unittest.TestCase):
    def run_converter(self, *args: str | Path) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            [sys.executable, str(CONVERTER), *map(str, args)],
            cwd=ROOT,
            text=True,
            capture_output=True,
            check=True,
        )

    def test_hatch_folder_converts_to_raw_atlas_package(self) -> None:
        with tempfile.TemporaryDirectory(prefix="petmui-test-") as temp:
            base = Path(temp)
            source = base / "source"
            output = base / "output"
            source.mkdir()
            sheet = Image.new("RGBA", (192 * 8, 208), (20, 30, 40, 255))
            sheet.save(source / "spritesheet.png")
            (source / "pet.json").write_text(
                json.dumps({"displayName": "Sample Pet", "spritesheetPath": "spritesheet.png"}),
                encoding="utf-8",
            )

            self.run_converter(source, output, "--rows", "idle")

            manifest = json.loads((output / "pet.json").read_text(encoding="utf-8"))
            self.assertEqual(manifest["renderer"], "raw-bgra-atlas-v1")
            self.assertEqual((output / "atlas.bgra").stat().st_size, 192 * 8 * 208 * 4)

    def test_static_image_converts_and_writes_relative_config(self) -> None:
        with tempfile.TemporaryDirectory(prefix="petmui-test-") as temp:
            base = Path(temp)
            source = base / "input.png"
            output = base / "pets" / "image-pet"
            config = base / "config.toml"
            Image.new("RGBA", (32, 48), (80, 90, 100, 255)).save(source)

            self.run_converter(source, output, "--static", "--write-config", config)

            manifest = json.loads((output / "pet.json").read_text(encoding="utf-8"))
            self.assertEqual(manifest["renderer"], "static-bgra-v1")
            config_text = config.read_text(encoding="utf-8")
            self.assertIn('pet_directory = "pets/image-pet"', config_text)
            self.assertIn("enable_typing_detection = false", config_text)

    def test_write_config_rejects_external_pet_output(self) -> None:
        with tempfile.TemporaryDirectory(prefix="petmui-test-") as temp:
            base = Path(temp)
            source = base / "input.png"
            output = base / "external-pet"
            config = base / "config.toml"
            Image.new("RGBA", (32, 48), (80, 90, 100, 255)).save(source)

            result = subprocess.run(
                [sys.executable, str(CONVERTER), str(source), str(output), "--static", "--write-config", str(config)],
                cwd=ROOT,
                text=True,
                capture_output=True,
            )

            self.assertNotEqual(result.returncode, 0)
            self.assertIn("local pets folder", result.stderr + result.stdout)

    def test_write_config_preserves_existing_settings(self) -> None:
        with tempfile.TemporaryDirectory(prefix="petmui-test-") as temp:
            base = Path(temp)
            source = base / "input.png"
            output = base / "pets" / "image-pet"
            config = base / "config.toml"
            Image.new("RGBA", (32, 48), (80, 90, 100, 255)).save(source)
            config.write_text(
                "\n".join(
                    [
                        "pet_size = 128",
                        'pet_directory_backup = "keep-me"',
                        "enable_typing_detection = false",
                        "",
                    ]
                ),
                encoding="utf-8",
            )

            self.run_converter(source, output, "--static", "--write-config", config)
            config_text = config.read_text(encoding="utf-8")

            self.assertIn("pet_size = 128", config_text)
            self.assertIn('pet_directory_backup = "keep-me"', config_text)
            self.assertIn('pet_directory = "pets/image-pet"', config_text)

    def test_emoji_text_converts_to_static_package(self) -> None:
        with tempfile.TemporaryDirectory(prefix="petmui-test-") as temp:
            output = Path(temp) / "emoji-pet"

            self.run_converter("--emoji", "star", output)

            manifest = json.loads((output / "pet.json").read_text(encoding="utf-8"))
            self.assertEqual(manifest["renderer"], "static-bgra-v1")
            self.assertTrue((output / "image.bgra").exists())

    def test_manifest_asset_path_traversal_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory(prefix="petmui-test-") as temp:
            base = Path(temp)
            source = base / "source"
            output = base / "output"
            source.mkdir()
            (base / "outside.png").write_bytes(b"not an image")
            (source / "pet.json").write_text(
                json.dumps({"spritesheetPath": "../outside.png"}),
                encoding="utf-8",
            )

            result = subprocess.run(
                [sys.executable, str(CONVERTER), str(source), str(output)],
                cwd=ROOT,
                text=True,
                capture_output=True,
            )

            self.assertNotEqual(result.returncode, 0)
            self.assertIn("inside the source folder", result.stderr + result.stdout)

    def test_missing_manifest_spritesheet_falls_back_to_default_file(self) -> None:
        with tempfile.TemporaryDirectory(prefix="petmui-test-") as temp:
            base = Path(temp)
            source = base / "source"
            output = base / "output"
            source.mkdir()
            sheet = Image.new("RGBA", (192 * 8, 208), (20, 30, 40, 255))
            sheet.save(source / "spritesheet.png")
            (source / "pet.json").write_text(
                json.dumps({"displayName": "Sample Pet", "spritesheetPath": "missing.png"}),
                encoding="utf-8",
            )

            self.run_converter(source, output, "--rows", "idle")

            manifest = json.loads((output / "pet.json").read_text(encoding="utf-8"))
            self.assertEqual(manifest["renderer"], "raw-bgra-atlas-v1")


if __name__ == "__main__":
    unittest.main()
