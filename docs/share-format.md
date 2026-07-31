# Share Format Specification

This document specifies the binary encoding format used by the Secret Sharing app for both simple and nested Shamir's Secret Sharing shares. It is intended for developers, security auditors, and end users who need to understand or manually decode shares without the app.

## Overview

The app produces shares as **URL-safe base64** strings (RFC 4648 Section 5, with `=` padding). There are two formats:

| Format | First decoded byte | Description |
|--------|-------------------|-------------|
| **Simple** | `0x01`-`0xFF` (blahaj x-coordinate) | Raw blahaj `Share` bytes, no prefix. |
| **Nested** | `0x00` (magic marker) | Custom binary header + raw blahaj `Share` bytes. |

The two formats are mutually exclusive — the app rejects mixed sets on recovery.

All cryptographic operations use Shamir's Secret Sharing over **GF(2^8)** with the irreducible polynomial **x^8 + x^4 + x^3 + x^2 + 1** (`0x11D`), as implemented by the [`blahaj`](https://crates.io/crates/blahaj) crate (v0.6).

## Simple Share Format

A simple share is the raw bytes of a blahaj `Share`, base64-encoded with no prefix or metadata.

### Binary layout

```
+-------------------+--------------------------------------+
| Byte 0            | Byte 1 .. N                          |
+-------------------+--------------------------------------+
| x-coordinate (u8) | share data (one byte per secret byte)|
+-------------------+--------------------------------------+
```

- **x-coordinate** (1 byte): The evaluation point x in {1, ..., 255} used by blahaj. Never `0x00` (x=0 would reveal the secret). This is what distinguishes simple shares from nested shares.
- **share data** (N bytes): One byte per byte of the original secret. Each byte is a polynomial evaluation at x over GF(2^8).

**Total size:** N + 1 bytes, where N = length of the secret in bytes.

### Example

Secret `"Hello World!"` (12 bytes), threshold 2, 4 shares:

```
Share 0:  AR0UGMgRlTD5XNUsyw==
          |  |  |  |  |  |  |  |  |  |  |  |  |
          01 1d 14 18 c8 11 95 30 f9 5c d5 2c cb
          |  └─────────── 12 data bytes ──────────┘
          x-coordinate = 1

Share 1:  AuKHhDmTV5leLgP06A==
          02 e2 87 84 39 93 57 99 5e 2e 03 f4 e8
          |  └─────────── 12 data bytes ──────────┘
          x-coordinate = 2
```

### Recovery

Any `threshold` shares (e.g., any 2 of 4) are sufficient. Feed the decoded bytes to any Shamir library that operates over GF(2^8) with polynomial `0x11D`. The library performs Lagrange interpolation at x = 0 to recover the original secret bytes.

## Nested Share Format

A nested share prepends a custom binary header before the raw blahaj share bytes. The header encodes the group hierarchy metadata needed for multi-level recovery.

### Binary layout

```
+--------+--------+-------------------------+---------------+-------------------------+
| Byte 0 | Byte 1 | Level 0   Level 1  ...  |               |                         |
|        |        | (2 bytes) (2 bytes)     | leaf_thresh   | raw blahaj share bytes  |
+--------+--------+-------------------------+---------------+-------------------------+
| 0x00   | depth  | thr  idx  thr  idx  ... | leaf_thresh   | x-coord  share_data ... |
| (u8)   | (u8)   | (u8)(u8)  (u8)(u8)      | (u8)          | (u8)     (M bytes)      |
+--------+--------+-------------------------+---------------+-------------------------+

Header size = 2 + depth * 2 + 1 bytes
Total size  = header_size + raw_share_size
```

### Field definitions

| Offset | Size | Field | Description |
|--------|------|-------|-------------|
| 0 | 1 | **magic** | Always `0x00`. Distinguishes nested from simple shares. |
| 1 | 1 | **depth** | Number of nesting levels (number of threshold/group-index pairs). Range: 1-255. |
| 2 | 2 * depth | **levels** | For each level *i* (0 = topmost): `threshold[i]` (u8) and `group_index[i]` (u8). The group_index is the blahaj x-coordinate of the share at that level — it identifies which group this share belongs to. |
| 2 + 2 * depth | 1 | **leaf_threshold** | The threshold for the innermost (leaf) group. |
| 3 + 2 * depth | variable | **raw_bytes** | Raw blahaj `Share` bytes: 1-byte x-coordinate + M data bytes. |

### Example: depth = 1 (flat groups)

Configuration: 2-of-3 top groups, each with 2-of-3 leaf shares.

```
A nested share (group 1, leaf share 1):

  Base64:  AAECAQIBc4M
  Decoded: 00 01 02 01 02 01 73 83 20

  Offset 0: 0x00  magic marker (nested share)
  Offset 1: 0x01  depth = 1
  Offset 2: 0x02  threshold[0] = 2 (need 2 of 3 groups)
  Offset 3: 0x01  group_index[0] = 1 (this share belongs to group at x=1)
  Offset 4: 0x02  leaf_threshold = 2 (need 2 of 3 leaf shares)
  Offset 5: 0x01  raw share x-coordinate = 1
  Offset 6: 0x73  \
  Offset 7: 0x83   > raw share data
  Offset 8: 0x20  /
```

### Example: depth = 2 (two levels of nesting)

Configuration: 2-of-2 top groups, each with 2-of-2 sub-groups, each with 2-of-3 leaf shares.

```
A nested share (group 1, sub-group 1, leaf share 1):

  Base64:  AAICAQIBAgHk6upF
  Decoded: 00 02 02 01 02 01 02 01 e4 ea ea 45

  Offset 0:  0x00  magic marker
  Offset 1:  0x02  depth = 2
  Offset 2:  0x02  threshold[0] = 2 (top: need 2 of 2 groups)
  Offset 3:  0x01  group_index[0] = 1 (top group x=1)
  Offset 4:  0x02  threshold[1] = 2 (sub-group: need 2 of 2 sub-groups)
  Offset 5:  0x01  group_index[1] = 1 (sub-group x=1)
  Offset 6:  0x02  leaf_threshold = 2 (need 2 of 3 leaf shares)
  Offset 7:  0x01  raw share x-coordinate = 1
  Offset 8:  0xe4  \
  Offset 9:  0xea   \
  Offset 10: 0xea    > raw share data
  Offset 11: 0x45  /
```

## Manual Decoding

The following pseudocode illustrates how to decode a share string and extract its metadata. This is illustrative — not tied to any specific language.

```
function decode_share(share_string):
    // Step 1: URL-safe base64 decode
    bytes = base64url_decode(share_string)

    // Step 2: Detect format by checking the magic byte
    if bytes[0] == 0x00:
        // --- Nested share ---
        depth = bytes[1]
        offset = 2

        thresholds = []
        group_indices = []

        // Step 3: Parse the header — one (threshold, group_index) pair per level
        for i in 0 .. depth:
            thresholds[i] = bytes[offset]
            group_indices[i] = bytes[offset + 1]
            offset += 2

        leaf_threshold = bytes[offset]
        offset += 1

        // The remainder is the raw blahaj share
        raw_share = bytes[offset..]

        return {
            type: "nested",
            depth: depth,
            thresholds: thresholds,
            group_indices: group_indices,
            leaf_threshold: leaf_threshold,
            raw_share: raw_share,
        }
    else:
        // --- Simple share ---
        // The entire buffer is the raw blahaj share
        // bytes[0] is the x-coordinate, bytes[1..] is the share data
        return {
            type: "simple",
            x_coordinate: bytes[0],
            raw_share: bytes,
        }
```

### Extracting the raw share

For simple shares, the entire decoded buffer is the raw blahaj share.

For nested shares, skip the header (`3 + depth * 2` bytes) — the remainder is the raw blahaj share. Feed these raw bytes to any Shamir library that operates over GF(2^8) with polynomial `0x11D`.

## Nested Recovery Process

Nested recovery proceeds **bottom-up**: leaf shares are combined within each group, then the recovered intermediate shares are combined at the next level, and so on until the original secret is obtained.

### Algorithm

Given a set of nested shares all with the same `depth`:

1. **Group** shares by their `group_indices` path. Two shares belong to the same leaf group if and only if their `group_indices` arrays are identical.

2. **Leaf recovery**: For each leaf group, if the group has >= `leaf_threshold` shares, perform Lagrange interpolation on the raw blahaj share bytes to recover the parent share. The parent share's path = the group's path with the last element removed.

3. **Repeat upward**: Group the recovered shares by their (shorter) path. If a group has >= the threshold for that level, recover the grandparent share.

4. **Final step**: At the top level, if there are >= `thresholds[0]` (top threshold) recovered shares, perform one final Lagrange interpolation to recover the original secret.

### Visual example (depth = 1)

```
Config: 2-of-3 top groups, each 2-of-3 leaf shares

         +--- Group 1 (x=1) ---+  +--- Group 2 (x=2) ---+  +--- Group 3 (x=3) ---+
         | L1a  L1b  L1c       |  | L2a  L2b  L2c       |  | L3a  L3b  L3c       |
         | (need 2 of 3)       |  | (need 2 of 3)       |  | (need 2 of 3)       |
         +---------+-----------+  +---------+-----------+  +---------+-----------+
                   |                       |                       |
              Recover x=1             Recover x=2             (not needed)
                   |                       |
                   +-----------+-----------+
                        Recover secret (need 2 of 3 groups)
                               |
                          Secret recovered
```

### What "enough shares" means

The access structure is hierarchical. For depth = 1 with top threshold T and leaf threshold L:

- You need >= T groups to each have >= L leaf shares.
- Having many shares in one group does not help if fewer than T groups are represented.
- Having shares from T groups but fewer than L shares in any one of them is insufficient.

## Security Considerations

### What the format reveals

The nested header is **metadata**, not secret material. It reveals:
- The group structure (how many levels, thresholds at each level).
- Which group a given share belongs to (via the group_index/x-coordinate).
- The leaf threshold.

This is by design: the share holder needs to know which shares to collect. The header does **not** reveal:
- The secret itself.
- Any individual share's data (which is the actual secret material).
- The total number of shares in a group (only the threshold).

### Information-theoretic security

Shamir's Secret Sharing provides **perfect secrecy**: any set of fewer than `threshold` shares reveals **zero information** about the secret. This holds for each level of nesting independently. An attacker who collects leaf shares from fewer than `leaf_threshold` members of a group learns nothing about that group's intermediate share, and therefore nothing about the secret.

### Magic byte collision

The `0x00` magic byte cannot collide with a simple share because blahaj uses x-coordinates starting at 1 (x = 0 would evaluate the polynomial at the origin, revealing the secret). The app's `nested_combine` command explicitly checks for mixed simple/nested share sets and rejects them.

### No integrity protection

The format has **no authentication or checksum**. A tampered share will either fail to decode (wrong header) or produce a wrong secret (modified data bytes). There is no way to detect which case occurred without additional out-of-band verification. If integrity is needed, consider wrapping shares with a MAC before splitting.

### Secret must be valid UTF-8

The app's `simple_combine` and `nested_combine` commands call `str::from_utf8` before returning the result. Binary secrets that are not valid UTF-8 will fail at recovery time, even with correct shares.

## References

| Resource | Description |
|----------|-------------|
| [`blahaj` crate (v0.6)](https://crates.io/crates/blahaj) | Rust implementation of Shamir's Secret Sharing over GF(2^8). |
| [Shamir, A. (1979). "How to Share a Secret"](https://doi.org/10.1145/359168.359176) | Original paper. |
| [RFC 4648 Section 5](https://datatracker.ietf.org/doc/html/rfc4648#section-5) | Base64 Encoding with URL and Filename Safe Alphabet. |
| `src-tauri/src/lib.rs` | App source: `encode_share` (line 158), `decode_share_from_bytes` (line 170), `do_split` (line 90), `nested_combine` (line 204). |
| `app/utils/nested.ts` | TypeScript mirror: `parseShare` (line 29), `base64UrlDecode` (line 14). |
