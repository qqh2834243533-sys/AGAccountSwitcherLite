# AG Account Switcher Lite

- `main` is the active Rust + Tauri 2 codebase. `archive/dotnet-simulation-prototype` is a historical archive.
- Keep account and recovery bytes inside the Windows vault boundary. Preserve the visible-window close gate, explicit account switching, empty Tauri capabilities, and CSP.
- Keep local configuration, tooling, credential data, and build outputs out of Git.
- Run `cargo fmt --all -- --check`, `cargo clippy --locked --all-targets -- -D warnings`, and `cargo test --locked`. The ignored live-client tests require explicit authorization and isolation.
- See the [build guide](docs/development/build.md) and [release guide](docs/development/release.md). Check the [GitHub Releases page](https://github.com/qqh2834243533-sys/AGAccountSwitcherLite/releases) for the published baseline.
