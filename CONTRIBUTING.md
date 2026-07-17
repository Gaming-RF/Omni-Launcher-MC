# Contributing to OmniLauncherMC

## Development Setup

1. Fork and clone the repository
2. Install dependencies: `pnpm install`
3. Run dev server: `pnpm tauri dev`

## Code Style

- **TypeScript**: Strict mode, ESLint, Prettier
- **Rust**: `clippy` with default lints, `rustfmt`
- **Commits**: Conventional Commits (`feat:`, `fix:`, `docs:`, etc.)

## Pull Request Process

1. Create a feature branch from `main`
2. Make your changes
3. Run `pnpm lint` and `cargo clippy`
4. Open a PR with a clear description

## Architecture

- Frontend communicates with Rust backend via Tauri's `invoke()` IPC
- Each `#[tauri::command]` in `src-tauri/src/commands/` maps to a TypeScript wrapper in `src/lib/tauri.ts`
- SQLite database is initialized on app start, stores accounts, instances, and installed mods
