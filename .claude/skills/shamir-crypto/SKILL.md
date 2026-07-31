---
name: shamir-crypto
description: Reference for the Shamir's Secret Sharing crypto logic in src-tauri/src/lib.rs. Covers the blahaj crate API (Sharks, Share, dealer/recover), URL-safe base64 share encoding, threshold/shares constraints, and how to modify the simple_split and simple_combine Tauri commands.
---

## What this covers

The crypto layer lives entirely in `src-tauri/src/lib.rs`. Two Tauri commands:

- `simple_split(secret: &str, threshold: u8, shares: u8) -> Result<Vec<String>, String>` — splits a secret into `shares` base64-encoded shares, requiring `threshold` shares to recover.
- `simple_combine(shares: Vec<String>) -> Result<String, String>` — recovers the original secret from a set of shares.

## blahaj crate API

The `blahaj` crate (v0.6) implements Shamir's Secret Sharing over GF(256).

### Key types

- `Sharks(threshold)` — constructor. Takes the threshold (minimum shares needed to recover) as a `u8` (1–255).
- `Share` — an opaque share type. Convert via:
  - `Vec::from(&share)` — Share → `Vec<u8>` (for encoding)
  - `Share::try_from(&bytes[..])` — `&[u8]` → `Share` (for decoding, returns `Result`)
- `sharks.dealer(secret: &[u8])` — returns an iterator of `Share`s. The threshold is set at `Sharks` construction time, not here.
- `sharks.recover(&[Share])` — returns `Result<Vec<u8>, Error>`. Must be called with `Sharks(n)` where `n` = number of shares provided (not the original threshold).

### Split flow

```
Sharks(threshold) → dealer(secret_bytes) → take(shares_count) → encode each as URL_SAFE base64
```

### Combine flow

```
decode each share from URL_SAFE base64 → Share::try_from(bytes) → Sharks(share_count) → recover(shares) → UTF-8 string
```

## Share encoding format

Shares are encoded as **URL-safe base64** (no padding) strings using `base64::engine::general_purpose::URL_SAFE`. Example share: `"AR0UGMgRlTD5XNUsyw=="`.

The raw bytes of a `Share` include a 1-byte share index (the x-coordinate) followed by the share data (one byte per byte of the secret). This is blahaj's internal format — don't construct shares manually.

## Constraints (validated in `simple_split`)

1. `threshold >= 1` — error: `"invalid threshold: 0"`
2. `shares >= 1` — error: `"invalid shares: 0"`
3. `threshold <= shares` — error: `"threshold can't be bigger than shares"`
4. Both `threshold` and `shares` are `u8` (max 255). The frontend enforces `.max(255)` in Zod schemas.

## Gotchas

- `simple_combine` creates `Sharks(len)` where `len` is the **number of shares provided**, not the original threshold. blahaj's `recover` needs at least `threshold` shares but accepts any count ≥ threshold. Passing fewer than the original threshold returns an error from `recover`.
- The secret must be valid UTF-8 — `simple_combine` calls `str::from_utf8` before returning. Binary secrets would fail here.
- `Shares` are NOT deterministic — each split produces different shares. The test in `test_simple_combine` uses hardcoded shares that are valid for the secret `"Hello World!"` with threshold 2.
- The `test_simple_flow_loop` test intentionally skips ~80% of iterations (`rand::random::<u8>() < 204`) for speed. This is by design, not flakiness.
- All error handling returns `String` errors (not custom error types) — Tauri serializes `Result<T, String>` to the frontend as `{ error: "message" }` on the `Err` variant.

## Testing

Run Rust tests from `src-tauri/`:

```bash
cargo test                    # all tests
cargo test test_simple_split  # single test
cargo test test_simple_flow   # the loop + random tests
```

Tests use `rand` (dev-dependency) for randomized round-trip testing across threshold/shares combinations and random secrets.
