# AGENTS.md

## What this is

Shamir's Secret Sharing desktop app. Nuxt 4 + Tauri 2 (Rust backend).

## Architecture

- **Frontend** (`app/`): Nuxt 4 SPA (`ssr: false`). Uses `@nuxt/ui`, Tailwind CSS, Zod. Single page (`app/pages/index.vue`) with Encrypt/Decrypt tabs. Calls Tauri commands via `invoke()` from `@tauri-apps/api/core`.
- **Backend** (`src-tauri/src/`): Rust. All crypto logic lives here, not in TS. Uses the `blahaj` crate (Shamir's Secret Sharing) and `base64` (URL-safe) for share encoding. Two Tauri commands: `simple_split` (split secret into shares) and `simple_combine` (recover secret from shares), defined in `src-tauri/src/lib.rs`.

## Commands

```bash
npm install              # also runs `nuxt prepare` (postinstall) to generate .nuxt/ types
npm run tauri dev       # full app dev (starts Nuxt dev server + Tauri window)
cargo test              # run Rust unit tests (from src-tauri/)
npm run generate        # static build to dist/ (used by Tauri build)
npm run tauri build     # production build
```

`npm run dev` alone starts only the web frontend — Tauri commands (`invoke`) will fail without the Rust backend running. Always use `npm run tauri dev` for the full app.

## Gotchas

- Dev server uses `strictPort: true` on port 3000; HMR uses port 5183 (ws protocol, `0.0.0.0` for mobile). Both ports must be free.
- Tauri build flow: `nuxt generate` produces `dist/` → Tauri bundles it (configured in `tauri.conf.json`, `frontendDist: "../dist"`).
- Rust tests in `lib.rs` (`test_simple_flow_loop`) intentionally skip ~80% of iterations for speed — this is by design, not flakiness.
- No lint, typecheck, or formatter scripts are configured. `vue-tsc` and `typescript` are installed as devDeps but no script wraps them.
- Env vars must be prefixed with `VITE_` or `TAURI_` to reach the frontend.
