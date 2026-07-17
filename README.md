# OmniLauncherMC

<p align="center">
  <strong>Open-source Minecraft launcher unifying Modrinth, CurseForge, and custom modpacks.</strong>
</p>

<p align="center">
  <a href="#features">Features</a> ·
  <a href="#getting-started">Getting Started</a> ·
  <a href="#build">Build</a> ·
  <a href="#architecture">Architecture</a> ·
  <a href="#contributing">Contributing</a>
</p>

---

## Why OmniLauncherMC?

Most Minecraft launchers lock you into a single mod platform. OmniLauncherMC lets you **search, install, and manage mods from Modrinth and CurseForge in the same instance** — pick whichever source has the version you need.

## Features

- **Multi-source mod management** — Install mods from Modrinth and CurseForge into the same instance
- **All mod loaders** — Fabric, Quilt, Forge, NeoForge, and Vanilla
- **Offline mode** — Play without a Microsoft account (just enter a username)
- **Modpack support** — Import `.mrpack` (Modrinth) and CurseForge modpack zips
- **Instance management** — Create, edit, duplicate, import/export, search/filter/sort
- **Java auto-download** — Automatically downloads the right Java version for your MC version
- **Process manager** — Track running games, view live console logs, kill processes
- **Download progress** — Real-time progress tracking with toast notifications
- **Keyboard shortcuts** — Ctrl+N (new), Ctrl+K (search), Ctrl+L (launch)
- **Cross-platform** — Windows, macOS, Linux (built with Tauri 2.x)

## Tech Stack

| Layer | Technology |
|-------|------------|
| Frontend | React 19, TypeScript, Tailwind CSS 4, Zustand 5, Vite 6 |
| Backend | Rust, Tauri 2.x |
| Database | SQLite (rusqlite, bundled) |
| APIs | Modrinth v2, CurseForge CFCore, Mojang Meta |

## Getting Started

### Prerequisites

- [Rust](https://rustup.rs/) 1.70+
- [Node.js](https://nodejs.org/) 20+
- [pnpm](https://pnpm.io/) 9+
- **Linux only:**
  ```bash
  sudo apt install libwebkit2gtk-4.1-dev libsoup-3.0-dev libjavascriptcoregtk-4.1-dev libssl-dev
  ```

### Install & Run

```bash
git clone https://github.com/Gaming-RF/Omni-Launcher-MC.git
cd Omni-Launcher-MC
pnpm install
pnpm tauri dev
```

### Build Release

```bash
pnpm tauri build
```

The built binary will be in `src-tauri/target/release/`.

## Architecture

```
OmniLauncherMC/
├── src/                          # React frontend
│   ├── pages/                    # Home, Discover, Settings, InstanceDetail
│   ├── components/               # UI components (InstanceCreator, ShareDialog, etc.)
│   ├── stores/                   # Zustand stores (instances, auth, settings, notifications)
│   ├── hooks/                    # Custom hooks (useActiveAccount, useKeyboardShortcuts, etc.)
│   └── lib/tauri.ts              # Type-safe Tauri IPC wrappers
├── src-tauri/                    # Rust backend
│   └── src/
│       ├── commands/             # Tauri IPC commands (minecraft, instances, loaders, settings, process)
│       ├── api/                  # API clients (modrinth, curseforge, minecraft, auth, loaders/)
│       ├── db/                   # SQLite (instances, accounts, mods, settings, migrations)
│       └── utils/                # launcher, process_manager, progress, download, java, modpack, paths
├── .github/workflows/ci.yml     # GitHub Actions CI
└── package.json
```

### Data Flow

```
Frontend (React) → Tauri IPC → Commands → API clients / DB → Response
                                   ↓
                          Process Manager → game-log events → Frontend
```

## Configuration

### CurseForge API Key

To browse and install CurseForge mods, you need a free API key:
1. Go to https://console.curseforge.com/
2. Create an account and generate an API key
3. Enter it in Settings → CurseForge API Key

### Microsoft Account (Optional)

For online multiplayer, register an Azure AD app:
1. Go to https://portal.azure.com/#blade/Microsoft_AAD_RegisteredApps
2. Register a new app (Personal Microsoft accounts only)
3. Enable "Allow public client flows"
4. Copy the Client ID into Settings

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/my-feature`)
3. Commit with conventional commits (`feat:`, `fix:`, `docs:`)
4. Push and open a Pull Request

## License

[MIT](./LICENSE)
