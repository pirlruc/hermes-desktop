# AI agent handoff — hermes-desktop

Tauri 2 desktop app. Rust core, React/TypeScript UI, Node 24. Pins a Delphi Engine release asset for the OpenAPI document. No submodule or disk path to delphi-engine.

## Pins

| Item | Value |
|------|-------|
| guardrails | 1.15.0 (`2d4bb70e95449f6922781716857adeb22badf676`) |
| github-scaffold | 1.12.0 (`9c443ea978851768f451b1df2020dd86dfb1a967`) |
| nodedevops | `52d746e815d2e7d193fed55d2a232c600f9222d7` |
| rustdevops | 1.0.5 (`03c252f48142e39e145d616240c47a7c7747e362`) |

## Commands

```bash
sh scripts/check-submodule-pins.sh
cargo test -p hermes-core
npm test -- --coverage --coverage.reporter=json-summary
```

Rust 1.89. `rusqlite` 0.41 bundles SQLite 3.54 (RUSTSEC-2026-0459). Tauri stays 2.2.5; that crate still builds on 1.89. `src-tauri` is its own workspace so `rust-quality` does not compile WebKit. The Linux bundle is `npm run tauri build -- --bundles deb` inside `tauri-build`.

## Known pitfalls

- Analog submodules are private. Checkout does not clone them without a read token.
- `check-guardrails-packs.py` does not read `docs/guardrail-deviations.yml`. A `Cargo.toml` fails that check until a rust pack exists. The CI-021 deviation records the missing thresholds file and points at guardrails issue 505.
- Host has no Rust or webkit2gtk. Build in the rustdevops image. `tauri-build` 1.0.1 installs `libxdo-dev`; an older pin fails the Linux link with `cannot find -lxdo`.
- Custom commands need `src-tauri/permissions/app.toml`. The capability identifiers are not inferred from `#[tauri::command]`.
- Coverage floors read `coverage/coverage-summary.json` totals. Exclude `src/**/*.tsx`; the shell is not in the 95/95 floor.
- `deny.toml` allows `Zlib` (foldhash via rusqlite 0.41) and `CDLA-Permissive-2.0` (ureq's webpki-roots). cargo-deny 0.21.0 loaded advisory-db; those two licenses were the remaining rejection.

## Suggested next work

1. Wait for `.github/workflows/ci.yml` on `feature-sync-core` to go green, then tag Linux 0.1.0. Vitest is 4.1.12 so `npm audit --audit-level=high` is clean; remaining findings are moderate.
2. HD-006 stays open. Cloud adapters are not in this slice.

*Last updated: 2026-11-21 — rustdevops 1.0.5; deny.toml allows Zlib and CDLA-Permissive-2.0.*
