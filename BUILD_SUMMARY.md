# OmniLauncherMC — Build Summary (July 2026)

## Overview

Open-source, cross-platform Minecraft launcher built with Tauri 2.x (Rust backend + React/TypeScript frontend). Unifies Modrinth, CurseForge, and custom modpacks into a single launcher.

- **Repo**: https://github.com/Gaming-RF/Omni-Launcher-MC
- **License**: MIT
- **Stack**: Tauri 2.x, Rust 1.96.1, React 18, TypeScript, Vite 6, Zustand 5, Tailwind CSS 3

## Architecture

```
src-tauri/src/
├── api/            # HTTP clients (Modrinth, CurseForge, Minecraft, Fabric, Forge, etc.)
├── commands/       # Tauri IPC commands (56 total)
│   ├── auth.rs     # Microsoft OAuth + offline accounts (5 commands)
│   ├── instances.rs # CRUD + settings + sharing (9 commands)
│   ├── loaders.rs  # Fabric/Quilt/Forge/NeoForge + mod management (20 commands)
│   ├── minecraft.rs # Launch, search, modpacks, packs (15 commands)
│   ├── java.rs     # Auto Java install (3 commands)
│   └── process.rs  # Game process monitoring (4 commands)
├── db/             # SQLite via rusqlite (accounts, instances, mods, settings)
└── utils/          # Paths, launcher, progress events, modpack parsing, process manager

src/
├── pages/          # Home, Discover, InstanceDetail, Settings
├── components/     # InstanceCreator, GameConsole, ModsTab, PacksTab, ShareDialog, Skeleton, PageTransition
├── stores/         # Zustand stores (auth, instances, i18n)
├── hooks/          # useActiveAccount
└── lib/            # tauri.ts (IPC wrappers), i18n.ts (8 locales)
```

## Features Completed (v0.2.0)

### 1. Modpack Browsing + One-Click Install
- Dual-source search: Modrinth + CurseForge modpacks
- Version picker for Modrinth modpacks before install
- One-click install creates instance, downloads mods, installs loader automatically
- Progress events emitted during download/parse/install phases
- Temp files cleaned up after install

### 2. Mod Update Checker
- `check_mod_updates` command queries Modrinth for newer versions of installed mods
- Returns version comparison (installed vs latest) with download URLs
- UI shows update count badge and per-mod update banners in ModsTab

### 3. Tauri Auto-Updater
- `tauri-plugin-updater` configured with GitHub Releases endpoint
- UpdateChecker component in Settings: check → download → install → relaunch
- Version displayed via `__APP_VERSION__` global (from package.json)
- Endpoint: `https://github.com/Gaming-RF/Omni-Launcher-MC/releases/latest/download/latest.json`

### 4. UI Polish
- PageTransition: fade+slide entrance animation on all route changes
- Skeleton loaders: InstanceCardSkeleton, SearchResultsSkeleton, ModRowSkeleton
- Loading state on Home page shows 6 skeleton cards in grid

### 5. Crash Detection + Log Viewer
- 10 crash pattern detectors (OOM, Java version mismatch, mod compatibility, missing dependencies, network, GLFW, exit codes)
- Crash analysis banner with suggested fixes when non-zero exit code detected
- Log search/filter with match count display
- Auto-scroll, timestamp toggle, kill button for running games

### 6. Shader + Resource Pack Support
- `list_installed_packs`, `toggle_pack`, `delete_pack` commands
- PacksTab component: enable/disable (renames `.disabled` suffix), delete
- Separate "Resources" and "Shaders" tabs in InstanceDetail

### 7. Multi-Account Management with Skin Preview
- `switch_active_account` command (updates `last_used` timestamp)
- Settings shows all accounts with skin head renders from mc-heads.net
- Click to switch, trash icon to remove
- Offline mode works without any Microsoft account

### 8. Internationalization (i18n)
- 8 locales: EN, ES, PT, ZH, JA, RU, DE, FR
- 65+ translation keys covering all pages (nav, home, discover, settings, instance, common)
- Auto-detects browser language on first visit
- Persists locale choice to localStorage
- Language selector in Settings
- All UI components use `t()` calls — changing language immediately affects visible text

## Key Technical Details

### Database Schema (SQLite)
- `accounts` — uuid, username, access_token, refresh_token, skin_url, last_used
- `instances` — id, name, game_version, loader, loader_version, icon, java_args, allocated_memory_mb, created_at, last_played, play_time_secs
- `mods` — id, instance_id, mod_id, name, source, version, file_name, enabled
- `settings` — key/value store (memory_mb, java_path, curseforge_api_key, theme, locale)

### CSP (Content Security Policy)
- `connect-src`: Modrinth API, CurseForge API, Mojang auth, Microsoft OAuth, Fabric/Forge/Quilt meta, Adoptium, GitHub
- `img-src`: Modrinth CDN, CurseForge CDN, mc-heads.net, crafatar.com
- `script-src`: updater.github.com

### Tauri Capabilities
- Core: window management, events, path resolution
- Shell: default (open URLs)
- Updater: default

## What's Still Pending (by design)

### Requires Configuration
- **Azure AD Client ID** — `CLIENT_ID` in `auth.rs` is `"YOUR_CLIENT_ID_HERE"`. Offline mode works without it.
- **CurseForge API key** — user sets via Settings. Required for CurseForge mod downloads in modpacks.
- **Auto-updater signing keypair** — placeholder pubkey in `tauri.conf.json`. Generate real keypair for production releases.
- **App icons** — currently solid emerald green placeholders. Need proper 32x32, 128x128, icon.ico, icon.png.

### Code Quality Items
- **No unit tests** — `cargo test` runs 0 tests. Need tests for modpack parsing, version comparison, crash detection.
- **`java_installations` table** — created in schema but never queried (dead schema).
- **CurseForge modpack direct install** — throws "not yet supported" error. Modrinth modpacks work fully.
- **Error handling** — most Rust commands use `.map_err(|e| e.to_string())`. Could use structured error types.

### Enhancement Ideas
- Mod browser could support installing mods directly into instances (currently only modpacks get one-click install)
- Resource pack/shader download from CurseForge/Modrinth
- Instance groups/categories
- Play time tracking (schema exists, not incremented)
- Settings page could show Java installation status
- Keyboard shortcuts (Ctrl+K for search is implemented, could add more)

## Build & Development

```bash
# Frontend dev server
pnpm dev

# Full Tauri dev (frontend + Rust backend)
cargo tauri dev

# Production build
cargo tauri build

# Type check
npx tsc --noEmit

# Rust check + lint
cargo check && cargo clippy -- -D warnings

# Format
cargo fmt
```

## Environment Requirements
- Rust 1.96.1+
- Node.js 18+
- pnpm
- Tauri CLI 2.11.4+
- Linux: `libwebkit2gtk-4.1-dev`, `libayatana-appindicator3-dev`, `librsvg2-dev`
