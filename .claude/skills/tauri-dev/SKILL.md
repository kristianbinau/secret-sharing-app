---
name: tauri-dev
description: Reference for the Tauri 2 + Nuxt 4 integration in this project. Covers how Tauri commands are defined and called, the dev/build flow, port configuration, tauri.conf.json structure, and frontend-backend communication patterns.
---

## Architecture

Nuxt 4 SPA (`ssr: false`) served by Tauri 2's embedded asset server. The frontend calls Rust functions via Tauri's IPC (`invoke`), not HTTP.

## Command pattern

### Define a Tauri command (Rust)

```rust
#[tauri::command]
fn my_command(arg: &str) -> Result<String, String> {
    Ok(arg.to_string())
}
```

Register it in `src-tauri/src/lib.rs`:

```rust
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![simple_split, simple_combine, my_command])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### Call a command from the frontend (Vue/TS)

```ts
import { invoke } from "@tauri-apps/api/core";

const result = await invoke<string[]>("simple_split", {
  secret: "my secret",
  threshold: 3,
  shares: 5,
});
```

Argument names in the `invoke` object must match the Rust function parameter names (snake_case).

### Error handling

Rust `Result<T, String>` maps to:
- `Ok(T)` → resolved promise with the value
- `Err(String)` → rejected promise with the string as error

The frontend components (`SimpleEncrypt.vue`, `SimpleDecrypt.vue`) currently don't catch `invoke` errors. If you add error handling, wrap `invoke` calls in try/catch.

## Dev and build flow

### Development

```bash
npm run tauri dev
```

This runs `beforeDevCommand: "npm run dev"` (from `tauri.conf.json`) to start the Nuxt dev server on port 3000, then opens the Tauri window pointing at `http://localhost:3000`.

`npm run dev` alone starts only the web frontend — `invoke()` calls will fail without the Rust backend. Always use `npm run tauri dev` for the full app.

### Production build

```bash
npm run tauri build
```

This runs `beforeBuildCommand: "npm run generate"` which produces `dist/` (static SPA), then Tauri bundles it (configured via `frontendDist: "../dist"` in `tauri.conf.json`).

```bash
npm run generate  # static build to dist/ only (no Tauri bundling)
```

## Port configuration (nuxt.config.ts)

- Dev server: port 3000 with `strictPort: true` — must be free, won't auto-increment.
- HMR: port 5183, ws protocol, `0.0.0.0` host (for mobile dev).
- Both ports must be free or `npm run tauri dev` will fail.

## tauri.conf.json structure

Key fields in `src-tauri/tauri.conf.json`:

- `beforeDevCommand` / `beforeBuildCommand` — shell commands Tauri runs before starting dev/build.
- `devUrl` — URL Tauri loads during development (`http://localhost:3000`).
- `frontendDist` — path to the static build output (`../dist`).
- `app.windows[0]` — window config (title, size, resizable).
- `bundle` — installer/bundle config (icons, targets).

## Cargo.toml dependencies

- `tauri` v2 — core framework.
- `serde` / `serde_json` — serialization for command arguments and return types.
- `blahaj` v0.6 — Shamir's Secret Sharing (see the `shamir-crypto` skill).
- `base64` v0.23 — URL-safe share encoding.

Dev-dependency: `rand` v0.9 (for tests only).

## Gotchas

- The `envPrefix` in `nuxt.config.ts` is set to `['VITE_', 'TAURI_']` — env vars must use these prefixes to reach the frontend.
- `src-tauri/gen/` and `src-tauri/target/` are build artifacts (gitignored).
- `main.rs` has `#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]` to hide the console window on Windows in release builds — do not remove.
- The `#[cfg_attr(mobile, tauri::mobile_entry_point)]` attribute on `run()` enables mobile entry points — keep it even if only building for desktop.
- Tauri commands are synchronous in Rust but async from the frontend (return Promises).
- The frontend uses `@nuxt/ui` components (`UForm`, `UTextarea`, `UButton`, `UTabs`, etc.) and Zod for form validation.
