# AdlerIt

AdlerIt is a small native Windows GUI for computing Adler-32 checksums. It is a
single desktop executable: open it, type or paste text, and copy the checksum.

The app is intentionally quiet. It performs no network access, writes no
settings or history, and uses native rendering through `egui` rather than a
browser engine or web runtime.

## Features

- Live Adler-32 calculation from text typed into the desktop window
- Hex and decimal output with a UTF-8 byte count
- One-click copy of the hex checksum
- Follows the Windows light/dark theme with a fixed blue accent
- Bundled JetBrains Mono for checksum/input text
- No network access, telemetry, update checks, persisted state, or background
  activity

## Implementation

- Single-crate Rust GUI app (`adlerit`)
- `egui/eframe` desktop UI in `src/app.rs`
- Small in-tree Adler-32 implementation in `src/hash.rs`
- JetBrains Mono embedded directly in the executable

## Layout

```text
src/
  main.rs               GUI entry point
  app.rs                desktop UI
  hash.rs               Adler-32 calculation

assets/
  fonts/
    JetBrainsMono-Regular.ttf
    OFL.txt

Scripts/
  build-windows-gui.ps1 Windows release build helper
  generate-release-notes.sh
  version.ps1

quickstart.txt          user-facing usage handout
```

## App Data

AdlerIt stores nothing. It writes no settings, history, cache, or data files.
There is nothing to back up, clear, or migrate between versions.

## Security Notes

AdlerIt has a deliberately small surface area:

- It performs no network access at all: no update checks, telemetry, or outbound
  connections.
- It only reads the text you type or paste into the app window.
- It persists no state, so there is no settings or data file to tamper with.
- The Windows build runs an offline-capability guard that checks the dependency
  graph and scans the binary for network, URL-opening, and startup-registry
  capability markers.
- Release builds are scanned with Microsoft Defender and carry GitHub
  build-provenance attestations.

Adler-32 is a fast checksum for integrity and error detection, not a
cryptographic hash. Do not use it to verify authenticity against a malicious
party or to store or compare secrets.

## Build And Run

AdlerIt targets Windows. Release builds produce one portable executable:

```text
AdlerIt-Windows-x64.exe
```

Run locally during development:

```bash
cargo run
cargo test
```

Build the Windows release executable:

```powershell
Scripts\build-windows-gui.ps1
```

The Windows build targets 64-bit Windows:

```text
x86_64-pc-windows-msvc
```

If Rust says the target is missing, install it once:

```powershell
rustup target add x86_64-pc-windows-msvc
```

The build output is:

```text
dist\AdlerIt-Windows-x64.exe
dist\AdlerIt-Windows-x64.exe.sha256
```

## Releases

The `version` in `Cargo.toml` is the single source of truth for the AdlerIt
release version. Update it, let Cargo refresh `Cargo.lock`, commit both files,
then create a matching `vX.X.X` tag.

To publish a GitHub Release, push a version tag:

```bash
git tag v0.2.0
git push origin v0.2.0
```

Before tagging, run **Build release app** manually from the GitHub Actions page
with the intended `vX.X.X` tag. The dry run checks formatting, tests, Clippy,
the dependency audit, the offline-capability guard, the Windows package, and
the Microsoft Defender scan.

Verify a downloaded release artifact with the GitHub CLI:

```bash
gh attestation verify AdlerIt-Windows-x64.exe --repo Joshndroid/AdlerIt
```

## Appearance

The desktop app follows the operating system's light/dark theme and uses a fixed
blue accent.

## Bundled Fonts

AdlerIt bundles JetBrains Mono Regular for checksum and input text. The font is
licensed under the SIL Open Font License 1.1; see `assets/fonts/OFL.txt`.

## License

AdlerIt is released under the GNU Affero General Public License v3.0; see
`LICENSE`.
