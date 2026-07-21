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
├── commands/       # Tauri IPC commands (80 total)
│   ├── auth.rs     # Microsoft OAuth + offline accounts (6 commands)
│   ├── instances.rs # CRUD + settings + sharing (9 commands)
│   ├── loaders.rs  # Fabric/Quilt/Forge/NeoForge + mod management (20 commands)
│   ├── minecraft.rs # Launch, search, modpacks, packs (15 commands)
│   ├── java.rs     # Auto Java install (3 commands)
│   ├── process.rs  # Game process monitoring (4 commands)
│   └── launcher.rs # Launcher import + profile export (24 commands)
├── db/             # SQLite via rusqlite (accounts, instances, mods, settings)
└── utils/          # Paths, launcher, progress events, modpack parsing, process manager

src/
├── pages/          # Home, Discover, InstanceDetail, Settings, Import
├── components/     # InstanceCreator, GameConsole, ModsTab, PacksTab, ShareDialog, Skeleton, PageTransition, WorldsTab, InstanceHooks
├── stores/         # Zustand stores (auth, instances, i18n)
├── hooks/          # useActiveAccount
└── lib/            # tauri.ts (IPC wrappers), i18n.ts (8 locales)
```

## Features Completed (v0.2.0)

### 9. Launcher Import (MultiMC/PrismLauncher/ATLauncher)
- `scan_launcher_instances` command: detects installed launchers, scans their instance directories
- Parses `instance.cfg` (MultiMC/Prism), `instance.json` (ATLauncher) to extract name, game version, loader
- `import_launcher_instance`: copies instance, parses installed mod JARs, inserts into DB
- Import page with launcher type tabs, instance selection checkboxes, batch import
- Progress bar during import, success/error reporting per instance
- Added `Download` icon to sidebar nav linking to `/import`

### 10. Profile Export (`.zip`)
- `export_instance_profile` command: creates portable `.zip` with mods, config, resourcepacks, shaderpacks, saves
- `profiles/` directory created in app data for exports
- Export button in InstanceDetail header (Download icon)
- Auto-names file: `{InstanceName}_v{Version}.zip`

### 11. Instance Hooks (Pre/Post Launch Commands)
- `update_instance_hooks` command: stores pre/post launch shell commands per instance
- InstanceHooks component: editable text fields for pre_launch, post_launch, post_exit hooks
- "Hooks" tab in InstanceDetail
- Hooks stored in `instances` DB table (pre_launch_hook, post_launch_hook, post_exit_hook columns)
- Migration added for schema upgrade

### 12. World Management
- `list_worlds` command: scans `saves/` directory, parses `level.dat` for world name, last played, game mode
- `delete_world` command: recursively removes world directory
- WorldsTab component: lists worlds with name, game mode, last played date, delete button
- "Worlds" tab added to InstanceDetail

### 13. Play Time Tracking
- `increment_play_time` command: adds seconds to instance's `play_time_secs` column
- `play_time_secs` column added to instances table via migration
- Auto-increments on game launch (polling via process manager)

### 14. Keyboard Shortcut (Ctrl+K Command Palette)
- Home page listens for `Ctrl+K` / `Cmd+K` to focus search input
- Discover page listens for `Ctrl+F` / `Cmd+F` to focus search
- Prevents default browser behavior

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
- `instances` — id, name, game_version, loader, loader_version, icon, java_args, allocated_memory_mb, created_at, last_played, play_time_secs, pre_launch_hook, post_launch_hook, post_exit_hook
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

## Bugs Fixed During Validation

| Bug | Severity | Fix |
|-----|----------|-----|
| `accounts.last_used` column missing from schema | Critical | Added ALTER TABLE migration in `migrations.rs`, updated `get_all_accounts` ORDER BY |
| `useEffect` missing dependency array in Home.tsx | High | Added `[filteredSorted, launchGame]` deps |
| `filteredSorted` used before declaration | High | Moved `useMemo` above `useEffect` in Home.tsx |
| Tab variable `t` shadowed i18n `t()` function | Medium | Renamed loop var to `tabId` in InstanceDetail.tsx |
| i18n translations never called in UI components | Medium | Wired `t()` into Home, Discover, InstanceDetail, Settings |
| 10MB build artifacts committed to git | Low | Removed from git, added `dist-packages/` to `.gitignore` |

## What's Still Pending (by design)

### Requires Configuration
- **Azure AD Client ID** — `CLIENT_ID` in `auth.rs` is `"YOUR_CLIENT_ID_HERE"`. Offline mode works without it.
- **CurseForge API key** — user sets via Settings. Required for CurseForge mod downloads in modpacks.
- **Auto-updater signing keypair** — placeholder pubkey in `tauri.conf.json`. Generate real keypair for production releases.
- **App icons** — currently solid emerald green placeholders. Need proper 32x32, 128x128, icon.ico, icon.png.

### Code Quality Items
- **No frontend unit tests** — `cargo test` runs 3 Rust tests. Need tests for modpack parsing, version comparison, crash detection.
- **`java_installations` table** — created in schema but never queried (dead schema).
- **CurseForge modpack direct install** — throws "not yet supported" error. Modrinth modpacks work fully.
- **Error handling** — most Rust commands use `.map_err(|e| e.to_string())`. Could use structured error types.

### Enhancement Ideas
- Mod browser could support installing mods directly into instances (currently only modpacks get one-click install)
- Resource pack/shader download from CurseForge/Modrinth
- Instance groups/categories
- Settings page could show Java installation status
- More keyboard shortcuts beyond Ctrl+K / Ctrl+F
- Instance screenshot management (auto-capture, gallery)
- Modpack creation wizard (select mods → export as modpack)

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
