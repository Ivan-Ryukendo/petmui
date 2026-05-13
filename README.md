# petmui Beta

petmui is a tiny Windows-first desktop pet app. It runs offline by default, shows a transparent always-on-top pet window, reacts to foreground activity, and exposes a tray menu plus a small Settings window for pause, hide/show, reload, local imports, and folder access.

Status: **beta / pre-release**. Builds are published as pre-release versions until the app is stable enough for `v1.0.0`.

## Build

```powershell
cargo build --release
```

The executable will be at:

```text
target\release\lightweight_desktop_pet.exe
```

For local testing:

```powershell
cargo run
```

## What The Current Beta Does

- Draws a transparent desktop pet using raw Win32 APIs and no runtime framework.
- Lets you drag the pet around the desktop.
- Adds a tray icon with pause, hide/show, reload pet, settings folder, pets folder, import folder, and exit actions.
- Adds a native Settings window for pause/hide/reload, opening local folders, and importing staged pet sources.
- Detects recent input without recording typed text.
- Detects foreground coding-agent/tool processes, game processes, and offline music processes, including Pear Desktop/YouTube Music by default.
- Resolves states with simple priority: gaming, agent work, recent input, music, sleep, idle.
- Reads an optional `config.toml` placed next to the executable.
- Can load an optional local pet package from `pet_directory`, falling back to the built-in procedural pet if the package is missing or invalid.
- Can switch into click-through overlay mode during configured games, so the pet does not capture mouse input.

## External Pet Packages

Set a package directory in `config.toml`:

```toml
pet_directory = "pets/default"
```

Relative paths are resolved from the loaded `config.toml` when possible. A package uses a tiny runtime format so the app can stay dependency-free:

```text
pets/default/
  pet.json
  atlas.bgra
```

`pet.json` for the package:

```json
{
  "name": "Default Pet",
  "renderer": "raw-bgra-atlas-v1",
  "atlas": "atlas.bgra",
  "cellWidth": 192,
  "cellHeight": 208,
  "columns": 8,
  "rows": [
    "idle",
    "waving",
    "jumping",
    "failed",
    "review",
    "running-right",
    "running-left",
    "running"
  ]
}
```

`atlas.bgra` is raw 32-bit BGRA pixels with no header. Frames are laid out left-to-right by column, and rows are laid out top-to-bottom in the same order as `rows`. The expected byte length is:

```text
cellWidth * columns * cellHeight * rows.length * 4
```

The renderer nearest-neighbor scales the selected cell into the configured `pet_size` window. Hatch-pet's 192x208 cells and row names are supported by this layout.

For a single still image, petmui also supports a static package:

```text
pets/static-pet/
  pet.json
  image.bgra
```

```json
{
  "name": "Static Pet",
  "renderer": "static-bgra-v1",
  "image": "image.bgra",
  "width": 192,
  "height": 192
}
```

To convert a hatch-pet package into this runtime format:

```powershell
python tools\convert_hatch_pet.py <path-to-codex-pet-folder> pets\your-pet
```

Then set:

```toml
pet_directory = "pets/your-pet"
```

To import a Codex pet and write a matching `config.toml` in one step, place the converted package under the local `pets` folder:

```powershell
python tools\convert_hatch_pet.py <path-to-codex-pet-folder> pets\your-pet --write-config config.toml
```

To import a single image as a static pet:

```powershell
python tools\convert_hatch_pet.py path\to\image.png pets\image-pet --static --write-config config.toml
```

To render local emoji/text into a static pet package:

```powershell
python tools\convert_hatch_pet.py --emoji "star" pets\star-pet --write-config config.toml
```

Emoji rendering uses Pillow and local fonts only. On Windows it tries Segoe UI Emoji first; if Pillow or the installed font cannot render a color emoji cleanly, save the emoji as an image from another local tool and use the `--static` image import path instead. No converter mode makes network calls.

When `--write-config` is used, the converter expects output inside the local `pets` folder. The app also loads configured pet packages only from that local folder by default. Do not commit local imported pets. The repository ignores `pets/` and `config.toml` so private/custom pets stay local.

The Settings window can import from the local staging folder without editing config manually:

```text
pets/imports/
  my-source-pet/
    pet.json
    spritesheet.webp
```

Open **Settings...** from the tray, choose **Import Folder**, and petmui converts the first staged pet folder into a local package under `pets/`, updates only `pet_directory`, and reloads the pet. Static images can be staged in the same import folder and imported with **Import Image**. Emoji/text pets can be created directly from the Settings window. These UI imports call the local converter and do not enable typing detection. In the current beta, Settings imports require Python and Pillow to be installed locally; the pet itself still runs without them.

State row selection prefers these names:

```text
idle -> idle
typing -> typing, running, idle
agent working -> review, working, running, typing, idle
agent success -> success, waving, idle
agent failed -> failed, fail, idle
music playing -> music-playing, music, waving, idle
music paused -> music-paused, idle
gaming -> jumping, running-right, running, idle
sleeping -> sleeping, sleep, idle
```

## Creating Pets With Codex Or ChatGPT

Recommended Codex workflow:

```text
Use the hatch-pet skill to create a pet for petmui.

Goal:
- Create a Codex-compatible animated pet package.
- Keep the pet small, readable, chibi, pixel-art-adjacent, and transparent-background friendly.
- Produce the usual hatch-pet outputs: pet.json, spritesheet.webp or spritesheet.png, QA contact sheet, and preview videos.

Visual style:
- Compact mascot proportions.
- Thick 1-2 px dark outline.
- Limited palette.
- Flat cel shading.
- Expressive face.
- No text, no UI panels, no detached effects, no soft shadows, no glow, no scenery.

Required motions:
- idle
- waving
- jumping
- failed
- review
- running-right
- running-left
- running

After the pet is finalized, convert it for petmui with:
python tools\convert_hatch_pet.py <hatch-pet-package-folder> pets\<pet-slug>
```

Manual ChatGPT character prompt:

```text
Create a small desktop-pet mascot concept for a transparent animated sprite.

The character should be:
- small chibi mascot proportions
- pixel-art-adjacent, not realistic
- thick dark outline
- limited color palette
- flat cel-shaded
- readable at tiny sizes
- expressive face and simple silhouette

Avoid:
- text or logos
- UI elements
- scenery/backgrounds
- soft glow
- drop shadows
- detached sparkles/effects
- realistic fur/materials
- complex accessories

Design the character so it can support these animation states:
idle breathing/blink, waving, jumping, failed/sad, reviewing/focused,
running right, running left, busy/working, music reaction, gaming reaction,
sleeping.

Return:
1. A short pet name.
2. A one-sentence personality.
3. A stable visual description.
4. A color palette.
5. Notes for keeping the character consistent across sprite frames.
```

For best results, use ChatGPT to develop the character concept and Codex with `hatch-pet` to turn it into a validated sprite package.

## Privacy

The app does not record keystrokes, screenshots, clipboard contents, or network traffic. Keyboard detection is off by default; when `enable_typing_detection = true`, it only increments an in-memory counter from Windows key events so the pet can react to active typing. The keyboard hook is disabled while the pet is paused or hidden. Foreground process names and limited window titles are read locally for state detection and are not stored or transmitted. Spotify OAuth is intentionally not included in the current beta; music detection is offline process-based fallback.

For games, set `click_through_in_games = true` and add game executable names under `game_processes`. When one of those games is foreground, the pet stays visible but allows mouse input to pass through to the game.

## Next Steps

- Improve the settings UI beyond the current tray-driven settings folder.
- Add Windows GlobalSystemMediaTransportControls integration for richer offline media state.
- Add an optional Pear Desktop plugin bridge for reliable YouTube Music playback state and track metadata.
- Add optional Spotify OAuth for track metadata only.
- Add local status adapters for specific coding agents.
- Add a compact AI chat panel with provider adapters for OpenAI, Claude, Gemini, and local models.

See `ROADMAP.md` for the YouTube Music, gaming, and AI companion path.

## Build Toolchain Note

This project uses direct Win32 API calls. On the MSVC Rust target, release builds need Visual Studio Build Tools with the C++ workload and Windows SDK on `PATH`.

If you do not want to install those locally, push the project to GitHub and run the `Windows Build` workflow from the Actions tab. It builds on GitHub's Windows runner and uploads a `lightweight-desktop-pet-windows` artifact containing `pet.exe`, `pet.json`, `config.example.toml`, `README.md`, `tools/convert_hatch_pet.py`, and `SHA256SUMS.txt`.

## Releases

Beta builds are published as GitHub pre-releases, starting with `v0.x.x-beta.n`. A release is not considered stable until `v1.0.0`.

## Web Option

A web version is useful as a preview or pet designer, but it cannot fully replace the native desktop app. Browsers cannot reliably stay on top of games, create a system tray app, detect global typing, inspect foreground processes, or run as a real background companion. The recommended path is:

- native app for the real desktop pet
- web app for previewing sprites, settings, and pet behavior
- GitHub Actions for building the `.exe` without installing Visual Studio Build Tools locally
