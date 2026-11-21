# hermes-desktop

Tauri desktop app that syncs a local reference library with Delphi Engine.

The app reads a local Zotero or Qiqqa library, asks before any PDF upload, and talks to [Delphi Engine 0.1.0](https://github.com/pirlruc/delphi-engine/releases/tag/0.1.0). It does not read a Delphi Engine checkout.

The first release is a Linux bundle. Windows and macOS come later.

```bash
cargo test -p hermes-core
npm test
npm run tauri build -- --bundles deb
```
