# Repository Guidelines

## Project Structure & Module Organization

NClientT is a Tauri 2 application with a Vue 3/TypeScript frontend and a Rust backend.

- `src/views/` contains route-level screens; `src/components/` contains reusable UI.
- `src/stores/` holds Pinia domain state, and `src/composables/` holds reusable Vue/browser behavior.
- `src/api/` is the frontend gateway to Tauri commands and external services. Import through `@/api`.
- `src/types/` contains shared frontend contracts; `src/i18n/locales/` contains translations.
- `src-tauri/src/` contains Rust commands, persistence, networking, downloading, and export logic.
- `src-tauri/icons/`, `screenshoot/`, and `docs/` contain assets and documentation. Do not commit generated `dist/` or `src-tauri/target/` output.

See `docs/FRONTEND_ARCHITECTURE.md` and `docs/ARCHITECTURE.md` before changing module boundaries.

## Build, Test, and Development Commands

- `npm ci`: install dependencies from `package-lock.json`.
- `npm run dev`: run the frontend-only Vite server.
- `npm run tauri:dev`: run the complete desktop application in development mode.
- `npm run typecheck`: run strict Vue/TypeScript checking without emitting files.
- `npm run build`: type-check and produce the frontend bundle in `dist/`.
- `cargo check --manifest-path src-tauri/Cargo.toml`: quickly validate the Rust backend.
- `cargo test --manifest-path src-tauri/Cargo.toml`: run Rust unit and documentation tests.
- `npm run tauri:build`: create production application bundles.

## Coding Style & Naming Conventions

Use two-space indentation in Vue and TypeScript, double quotes, semicolons, and strict types; avoid `any`. Name Vue components in PascalCase (`GalleryCard.vue`), composables with `useCamelCase`, and stores by domain. Keep views focused on orchestration; move shared interactions into composables and backend transport into `src/api/`.

Follow standard Rust formatting with `cargo fmt --all`; use `snake_case` for modules/functions and `PascalCase` for types. Run `cargo fmt --all -- --check` before submitting Rust changes.

## Testing Guidelines

There is no frontend test runner or coverage threshold. Every change must pass `npm run build`, `cargo check`, and `cargo test`. Add Rust tests beside the module under `#[cfg(test)]`. For frontend tests, use `*.spec.ts` near the tested module and add the runner command to `package.json`.

## Commit & Pull Request Guidelines

Use concise, imperative subjects. Recent history favors Conventional Commit prefixes such as `feat:`, `fix:`, `refactor:`, and `chore:`. Keep unrelated changes in separate commits.

Pull requests should explain behavior and architecture changes, list verification commands, link relevant issues, and include screenshots or recordings for UI work. Note platform-specific impact (Windows, Linux, or Android) and never commit API keys, local settings, build artifacts, or generated timestamp files.
