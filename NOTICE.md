# Attribution & License Notice (CC BY-NC-SA 4.0)

**AG Account Switcher Lite (`ag-account-switcher-lite`)** is a derivative work licensed for **NonCommercial** use under the [Creative Commons Attribution-NonCommercial-ShareAlike 4.0 International Public License (CC BY-NC-SA 4.0)](https://creativecommons.org/licenses/by-nc-sa/4.0/). See [LICENSE](LICENSE) for the full legal text.

## Upstream Attribution Chain

1. **Direct Upstream**: Derived from [**anglee0323/agy-switch**](https://github.com/anglee0323/agy-switch) — specifically release [`v4.10.18`](https://github.com/anglee0323/agy-switch/releases/tag/v4.10.18) (commit `815714f9ef431233f663a3c9910071a683de7e5d`), licensed under `CC BY-NC-SA 4.0`.
2. **Original Upstream Credit**: `anglee0323/agy-switch` credits [**lbjlaq/Antigravity-Manager**](https://github.com/lbjlaq/Antigravity-Manager) as its original upstream inspiration and foundation.

## Summary of Modifications Made in AG Account Switcher Lite

Pursuant to Section 3(a)(1)(B) of `CC BY-NC-SA 4.0`, the following architectural and code modifications were made relative to `anglee0323/agy-switch v4.10.18`:

- **Standalone Windows-Only Tauri 2 Entry Point**: Replaced the multi-platform React/TypeScript/Zustand frontend and broad Tauri backend with a minimal, zero-framework HTML/CSS/JS window (`ui/`) and a dedicated Windows x64 Rust crate (`src/`).
- **Protected Four-Slot OS Credential Vault**: Removed plaintext `.antigravity_tools/accounts/*.json` token storage, import/export flows, third-party OAuth callback servers, and token-refresh network clients. Saved sessions (up to 4 slots) and rollback recovery state are stored exclusively in the current Windows user's Credential Manager (`CRED_TYPE_GENERIC`), using atomic in-place `CredWriteW` replacement and read-back verification.
- **Removal of Automated Polling & Client Injection**: Removed quota polling, usage/pricing dashboards, background account sync, automatic account rotation, CLI/SQLite `.vscdb` database rewriting, updater endpoints, and experimental client runtime injection.
- **Silent Window-Safe Shutdown & External Plugin Compatibility**: Replaced focus-stealing tray automation and unconditional force-termination with a two-phase shutdown gate (`WM_CLOSE` to visible windows first; aborts on timeout if any visible window remains open; cleans up remaining verified tray-resident processes and stale `DevToolsActivePort` only after all visible windows close), automatic write-back of official background-refreshed tokens to their matching slot, and sanitized relaunch preservation for official flags and Antigravity-affiliated companion launchers.

## Disclaimer & Trademarks

- **No Commercial Use**: Under `CC BY-NC-SA 4.0`, this software may **not** be used primarily for or directed toward commercial advantage or monetary compensation without separate permission from all applicable rights holders.
- **No Endorsement**: Neither the upstream authors (`anglee0323`, `lbjlaq`) nor Google endorse this derivative project or your use of it.
- **Independent Tool**: This project is independent of Google LLC. `Google` and `Antigravity` are referenced solely to identify the target desktop application. This software is provided "as-is" without warranties of any kind.
