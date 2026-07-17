# OmniLauncherMC

Open-source Minecraft launcher unifying Modrinth, CurseForge, and custom modpacks into a single, unified interface.

## Tech Stack

- **Frontend**: React 19 + TypeScript + Tailwind CSS 4 + Zustand
- **Backend**: Rust (Tauri 2.x)
- **Database**: SQLite (via rusqlite)
- **Targets**: Windows, macOS, Linux

## Getting Started

### Prerequisites

- [Rust](https://rustup.rs/) (1.70+)
- [Node.js](https://nodejs.org/) (20+)
- [pnpm](https://pnpm.io/) (9+)
- System dependencies (Linux only):
  ```bash
  sudo apt install libwebkit2gtk-4.1-dev libsoup-3.0-dev libjavascriptcoregtk-4.1-dev libssl-dev
  ```

### Install

```bash
pnpm install
```

### Development

```bash
pnpm tauri dev
```

### Build

```bash
pnpm tauri build
```

## Project Structure

```
OmniLauncherMC/
├── src/                    # React frontend
│   ├── components/         # Reusable UI components
│   ├── pages/              # Route pages (Home, Discover, Settings)
│   ├── stores/             # Zustand state stores
│   ├── hooks/              # Custom React hooks
│   └── lib/                # Tauri API wrappers
├── src-tauri/              # Rust backend
│   └── src/
│       ├── commands/       # Tauri IPC commands
│       ├── api/            # External API clients
│       ├── db/             # SQLite database
│       ├── models/         # Shared data models
│       └── utils/          # Download manager, file ops
└── package.json
```

## Roadmap

- **Phase 1 (MVP)**: Microsoft auth, Minecraft version download, instance management, game launch
- **Phase 2**: Modrinth + CurseForge integration, mod browser, modpack installer, mod loader support
- **Phase 3**: Auto-update, Java manager, CI/CD, polish

## Contributing

See [CONTRIBUTING.md](./CONTRIBUTING.md) for guidelines.

## License

MIT
