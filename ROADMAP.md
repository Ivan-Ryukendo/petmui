# Desktop Pet Roadmap

## v0.1 Native Companion

- Tiny Windows-first native app.
- Transparent always-on-top pet window.
- Tray controls for pause, hide/show, and exit.
- Offline activity detection:
  - typing without text capture
  - foreground coding tools/agents
  - manually configured games
  - Pear Desktop/YouTube Music process fallback

## v0.2 YouTube Music Bridge

Keep the pet native and small. Add a separate Pear Desktop plugin that reports playback state to the pet.

Recommended bridge:

- Pear plugin reads YouTube Music player state through Pear's renderer/plugin APIs.
- Plugin sends only small state events:
  - `playing`
  - `paused`
  - `track_changed`
  - `title`
  - `artist`
  - optional `like/dislike`
- Native pet receives events over localhost or a Windows named pipe.
- If the bridge is missing, the pet falls back to process detection.

This avoids bundling Electron into the pet and keeps the pet executable small.

## v0.3 Games

- v1 game behavior remains manual: add game executable names to `config.toml`.
- When a configured game is foreground, the pet always enters `gaming`.
- Later win/loss detection should be plugin-based per game, because generic win/loss detection usually requires game-specific logs, APIs, overlays, or computer vision.

## v0.4 AI Companion

Add a separate compact chat panel opened from the pet, not embedded directly inside the always-on-top sprite.

Provider model:

- `OpenAIProvider`
- `ClaudeProvider`
- `GeminiProvider`
- future local model provider

Recommended authentication:

- Use official API keys where available.
- Do not automate consumer subscription web UIs by default; it is brittle and may violate service terms.
- Store credentials in Windows Credential Manager, not plain config files.

Pet behavior:

- Short speech bubble for small replies.
- Larger docked panel for real conversations.
- The pet can react emotionally while the provider is thinking, succeeds, or fails.
