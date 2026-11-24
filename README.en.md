# AG Account Switcher Lite

[中文说明](README.md) · [Attribution & Notice (NOTICE)](NOTICE.md) · [Security Design](docs/security/design.md)

**AG Account Switcher Lite (`ag-account-switcher-lite`)** is an unofficial manual four-account switcher for the Windows 11 x64 Antigravity desktop app, built with Rust and Tauri 2.

It has no built-in OAuth server, no bundled client secrets, no quota polling, no model API proxy, and no workspace or conversation database writes. Initial sign-in and re-authentication are always handled by the official Antigravity app and official browser flow.

## Key Capabilities

- **Four-Slot Windows Credential Vault**: Save up to 4 accounts signed in through the official Antigravity app. Credentials and rollback state stay inside the current Windows user's Credential Manager (`CRED_TYPE_GENERIC`), while the WebView2 UI receives only slot numbers and labels.
- **Active Slot Detection & Refreshed Token Sync**: Highlights the currently active registered slot and automatically writes official background-refreshed tokens back to their matching slot before switching or restoring.
- **Window-Safe Shutdown Gate (1-A)**:
  - When Antigravity is closed: switches credentials and launches the app in a single click.
  - When Antigravity is running: prompts for confirmation first, then sends `WM_CLOSE` to visible windows without opening tray menus or stealing mouse/keyboard focus.
  - **Data-Loss Guard**: If any visible window remains open when the close timeout expires (for example, an unsaved-changes dialog), switching aborts safely without modifying credentials. Remaining verified tray-resident processes (checked by executable image path and process creation time) are terminated only after all visible windows have closed or hidden to the tray.
- **Plugin and launch-option compatibility.** Relaunch validates loopback debugging, local proxy, language, scaling, and selected rendering options. It can restart a companion observed during a running-client switch. A cold start opens Antigravity directly. Custom installation directories can be discovered through the known Antigravity installer registration or machine-wide Windows App Paths.
- **Dual-Layout Modern UI, Tray Residency & Quick Summon**: Features an Antigravity-inspired frameless rounded window (with DWM system border suppression and integrated custom titlebar controls), instant switching between **Matrix** and **List** layouts, light/dark themes, and an interactive radial starburst particle canvas that pauses at 0% CPU when hidden to the tray. Supports per-user silent startup (auto-healing portable path moves), customizable global shortcuts (`Alt+Shift+G` by default), `1`–`4` slot keys, and `Esc` to hide.
- **Verified Rollback Journal**: Verifies every credential write via read-back and automatically restores the previous state if launching Antigravity fails, with manual session rollback available when the client is closed.

## Usage

1. Open the official Antigravity desktop app and sign in through its browser flow. Confirm the expected account in Antigravity.
2. Open `ag-account-switcher-lite.exe`, click **Save current account**, and enter a label (up to 4 accounts).
3. Make sure no agents are actively running and save your work before clicking a saved account. Verify the active identity inside Antigravity after it opens.
4. If Google requests re-authentication, sign in using the official Antigravity flow. Remove and re-save a slot if its saved authorization has been revoked.

## Building & Packaging on Windows

Requires Windows 11 x64, Rust Stable (`x86_64-pc-windows-msvc`), MSVC C++ Build Tools, a Windows SDK, and the WebView2 Runtime. See [Build Guide](docs/development/build.md).

```powershell
cargo test --locked
cargo build --locked --release
./scripts/package.ps1
```

## Scope & Limitations

- **Shared Sign-In Surface**: This tool updates only the desktop app's `gemini:antigravity` Windows Credential Manager entry and does not rewrite `~/.gemini/antigravity-cli/antigravity-oauth-token` or `.vscdb`. Running `agy` CLI or Antigravity IDE processes block switching until closed.
- **Unofficial Integration**: Google does not publish a third-party multi-account switching contract for the standalone Windows desktop app. Review [Compliance & Risk Assessment](docs/security/compliance-assessment.md) before use.

## Source Attribution & License

Derived from [anglee0323/agy-switch v4.10.18](https://github.com/anglee0323/agy-switch/releases/tag/v4.10.18) (commit `815714f9ef431233f663a3c9910071a683de7e5d`), which credits [lbjlaq/Antigravity-Manager](https://github.com/lbjlaq/Antigravity-Manager).

Licensed for **NonCommercial** use under **[CC BY-NC-SA 4.0](LICENSE)** (Creative Commons Attribution-NonCommercial-ShareAlike 4.0 International). See [NOTICE.md](NOTICE.md) for the complete attribution chain and list of modifications. The earlier `.NET 10 WPF` offline simulation prototype is preserved on the `archive/dotnet-simulation-prototype` branch.

This is an independent tool and is not affiliated with, authorized, or endorsed by Google LLC.
