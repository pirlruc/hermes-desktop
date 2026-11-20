# AI agent handoff — hermes-desktop

Tauri 2 desktop app. Rust core, React/TypeScript UI, Node 24. Pins a Delphi Engine release asset for the OpenAPI document. No submodule or disk path to delphi-engine.

## Pins

| Item | Value |
|------|-------|
| guardrails | 1.15.0 (`2d4bb70e95449f6922781716857adeb22badf676`) |
| github-scaffold | 1.12.0 (`9c443ea978851768f451b1df2020dd86dfb1a967`) |
| nodedevops | `52d746e815d2e7d193fed55d2a232c600f9222d7` |
| rustdevops | `ccaeb9a183d1cad469cf6d55e715c19c6a55a767` |

## Commands

```bash
sh scripts/check-submodule-pins.sh
```

## Known pitfalls

- Analog submodules are private. Checkout does not clone them without a read token.
- `check-guardrails-packs.py` does not read `docs/guardrail-deviations.yml`. A `Cargo.toml` fails that check until a rust pack exists. The deviation points at guardrails issue 505.
- Host has no Rust or webkit2gtk. Build in the rustdevops image.

## Suggested next work

1. Tauri skeleton, sync engine, Zotero and Qiqqa readers, consent gate, cache, and the React UI (HD-002 through HD-005).
2. Callers for nodedevops and rustdevops, then a Linux 0.1.0 bundle.

*Last updated: 2026-11-20 — scaffold and HD-001..006 authored.*
