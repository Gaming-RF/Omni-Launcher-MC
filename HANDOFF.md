# OmniLauncherMC — Handoff Document

**Date:** 2026-07-22
**Repo:** https://github.com/Gaming-RF/Omni-Launcher-MC
**Branch:** `main` (only branch)
**Current version:** 0.2.2 (code bumped, tag pushed, **release is still DRAFT on GitHub**)

---

## 1. Project Overview

OmniLauncherMC is an open-source Minecraft launcher built with **Tauri 2.x** (Rust backend + React frontend). It unifies **Modrinth** and **CurseForge** mod sources into a single instance manager — users can install mods from either platform into the same Minecraft instance.

### Key capabilities (as of v0.2.2)

| Feature | Status |
|---------|--------|
| Multi-source mod search (Modrinth + CurseForge) | Done |
| All mod loaders (Fabric, Quilt, Forge, NeoForge, Vanilla) | Done |
| Microsoft OAuth login + offline mode | Done |
| Modpack import (.mrpack + CurseForge .zip) | Done |
| Modpack export (.mrpack) | Done |
| Instance CRUD, duplicate, search/filter/sort, groups | Done |
| Java auto-download (8, 16, 17, 21) | Done |
| Process manager + live console logs | Done |
| Download progress toasts | Done |
| Keyboard shortcuts (Ctrl+N, Ctrl+K, Ctrl+L) | Done |
| Cross-platform installers (Linux .sh, macOS, Windows .ps1) | Done |
| Tauri native auto-updater with signed updates | Done |
| Instance sharing via share codes | Done |
| Resource packs, screenshots, worlds tabs | Done |
| Graphics settings per instance | Done |
| Mod categorizer (client/server/both) | Done |
| Mirror/proxy selector (for China/restricted networks) | Done |
| Templates page (popular modpacks) | Done |
| Discover page (popular mods) | Done |

---

## 2. Tech Stack

| Layer | Technology | Version |
|-------|-----------|---------|
| Frontend framework | React | 19 |
| Language | TypeScript | 5.x |
| Styling | Tailwind CSS | 4 |
| State management | Zustand | 5 |
| Build tool | Vite | 6 |
| Desktop framework | Tauri | 2.x |
| Backend language | Rust | 1.70+ |
| Database | SQLite (rusqlite, bundled) | — |
| Package manager | pnpm | 9+ |
| External APIs | Modrinth v2, CurseForge CFCore, Mojang Meta | — |

---

## 3. Architecture

```
src/                          ← React frontend (TypeScript)
  App.tsx                     ← Router, layout, page transitions
  pages/                      ← Route-level pages (Home, Discover, Library, Settings, etc.)
  components/
    auth/                     ← Login, account selector
    common/                   ← Shared UI (Button, Modal, Toast, ProgressBar, etc.)
    instance/                 ← Instance creator, detail tabs, console, mods
    layout/                   ← Sidebar, GroupSidebar, RunningInstancesBar
    mods/                     ← Mod search/browse cards
    settings/                 ← MirrorSelector, settings panels
  stores/                     ← Zustand stores (auth, instances, settings, i18n, notifications)
  hooks/                      ← Custom hooks (useActiveAccount, useDownloadProgress, etc.)
  lib/                        ← i18n, tauri helpers

src-tauri/                    ← Rust backend
  src/
    api/
      auth.rs                 ← Microsoft OAuth + offline auth
      modrinth.rs             ← Modrinth v2 API client
      curseforge.rs           ← CurseForge CFCore API client
      minecraft.rs            ← Mojang meta, version manifest, asset/library download
      loaders/                ← fabric.rs, forge.rs, neoforge.rs, quilt.rs
    commands/                 ← 25 Tauri command modules (one per feature area)
    db.rs                     ← SQLite schema and queries
    launcher.rs               ← Minecraft process launch logic
    java.rs                   ← Java detection + auto-download
    instances.rs              ← Instance CRUD, import/export
    lib.rs                    ← Tauri app builder, command registration
  Cargo.toml
  tauri.conf.json             ← Tauri config (CSP, window, updater, bundle)
```

### Data flow

1. Frontend calls Tauri commands via `@tauri-apps/api` invoke
2. Rust commands in `src-tauri/src/commands/` orchestrate logic
3. API modules (`src-tauri/src/api/`) handle external HTTP calls (Modrinth, CurseForge, Mojang)
4. SQLite (`db.rs`) persists instances, settings, accounts, mod metadata
5. `launcher.rs` assembles JVM args and spawns the Minecraft process

---

## 4. Release State

### What's done

- **v0.2.2 is tagged** on `main` (`412febf`)
- **GitHub Release exists but is a DRAFT** — not visible on the public releases page yet
- Built assets are uploaded: `.deb`, `.rpm`, `.AppImage`, `.msi`, `.exe`, `.dmg` (both x64 and aarch64 for macOS)
- `latest.json` at repo root still points to **v0.2.1** (needs update when v0.2.2 is published)

### What's NOT done (release)

- [ ] **Publish the v0.2.2 draft release** on GitHub (make it non-draft)
- [ ] **Update `latest.json`** — currently has version `0.2.1`, needs to point to `0.2.2` with correct signatures and download URLs for all platforms (linux-x86_64, windows-x86_64, darwin-x86_64, darwin-aarch64)
- [ ] `latest.json` only has `linux-x86_64` platform — needs Windows and macOS entries added
- [ ] Old v0.2.1 assets are mixed into the v0.2.2 draft release — clean those up

### Installers

- `install.sh` (Linux/macOS) and `install.ps1` (Windows) are in repo root — they clone + build from source
- Pre-built packages in `dist-packages/` (rpm, deb) are stale v0.1.0 — don't rely on these

---

## 5. Key Files to Know

| File | Purpose |
|------|---------|
| `src-tauri/tauri.conf.json` | App config, CSP, updater settings, bundle config |
| `src-tauri/Cargo.toml` | Rust dependencies |
| `package.json` | Frontend dependencies, scripts |
| `src/App.tsx` | Main router and layout |
| `src/stores/instances.ts` | Core instance state management |
| `src/stores/auth.ts` | Auth state (accounts, active account) |
| `src-tauri/src/db.rs` | SQLite schema |
| `src-tauri/src/commands/mod.rs` | Registers all Tauri commands |
| `src-tauri/src/commands/instances.rs` | Instance CRUD commands |
| `src-tauri/src/commands/auth.rs` | Auth Tauri commands |
| `src-tauri/src/api/auth.rs` | Microsoft OAuth flow |
| `src-tauri/src/launcher.rs` | Minecraft launch logic |
| `src-tauri/src/java.rs` | Java auto-detection and download |
| `latest.json` | Auto-updater manifest (consumed by Tauri updater plugin) |

---

## 6. How to Run

```bash
# Install deps
pnpm install

# Dev mode (hot reload)
pnpm tauri dev

# Production build
pnpm tauri build

# Lint frontend
pnpm lint
```

### Linux system deps (Ubuntu/Debian)

```bash
sudo apt install libwebkit2gtk-4.1-dev libsoup-3.0-dev libjavascriptcoregtk-4.1-dev \
  libssl-dev libgtk-3-dev libappindicator3-dev patchelf librsvg2-dev
```

---

## 7. CSP (Content Security Policy)

Defined in `tauri.conf.json`. Already allows:
- Modrinth API, CurseForge API, Mojang endpoints, Microsoft auth endpoints
- Modrinth CDN images, CurseForge CDN images, mc-heads.net, crafatar.com
- GitHub API + usercontent (for updater)
- Adoptium API (for Java download)

If adding new external domains, update the `csp` field in `tauri.conf.json`.

---

## 8. Known Issues / Gotchas

1. **`latest.json` is stale** — still at v0.2.1, only has `linux-x86_64` platform
2. **v0.2.2 release is a draft** — needs manual publish on GitHub
3. **Mixed assets in draft release** — v0.2.1 and v0.2.2 binaries are both in the v0.2.2 draft
4. **`dist-packages/` has stale v0.1.0 files** — ignore these, CI builds fresh ones
5. **No automated CI/CD pipeline visible** — releases appear to be built and uploaded manually or via a GitHub Action that creates drafts
6. **`safe_html_output` regex bug mentioned in AGENTS.md** — this is for the Bolan project, not OmniLauncherMC (ignore)

---

## 9. Immediate Next Steps

1. **Publish v0.2.2 release** — go to GitHub releases, edit the v0.2.2 draft, uncheck "Draft", verify assets
2. **Update `latest.json`** — bump version to 0.2.2, add all platform entries (linux-x86_64, windows-x86_64, darwin-x86_64, darwin-aarch64) with correct signatures and URLs
3. **Clean up release assets** — remove old v0.2.1 files from the v0.2.2 release
4. **Test auto-updater** — install v0.2.1, verify it detects and downloads v0.2.2 update
5. **Consider automating releases** — a GitHub Action that builds all platforms and creates a release with `latest.json` would save manual work

---

## 10. Session Context

The previous conversation (~3700 messages, compressed) was a long development session that built most of the features listed above. The repeated messages "it only a draft it not in the release page yet" were about the v0.2.2 GitHub release being stuck in draft state — this is still unresolved.

The `main` branch is clean (no uncommitted changes). All work has been committed and pushed.
