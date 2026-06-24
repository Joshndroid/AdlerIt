# AdlerIt

AdlerIt is a small native Rust app for computing Adler-32 checksums. A single executable provides both a desktop interface and a command-line interface: launch it with no arguments for the window, or pass flags to hash text, a file, or standard input from a terminal.

The app is focused by design: it performs no network access, writes no files of
any kind, and uses native rendering through `egui` rather than a browser engine
or web-based application runtime.

## Features

- Single binary that is both the desktop app and the command-line tool
- Live desktop calculator showing the Adler-32 value as hex and decimal
- One-click copy of the hex checksum from the desktop app
- UTF-8 byte count for the current input
- Command-line hashing of inline text, a file's raw bytes, or standard input
- Selectable CLI output format: hex, decimal, or both
- Light, dark, and system themes with a single Material flat blue accent
- Bundled JetBrains Mono font for consistent rendering
- Low-footprint native Rust/egui desktop app
- No network access, telemetry, update checks, persisted state, or background
  activity

## Implementation

- Single-crate Rust binary (`adlerit`) targeting desktop and CLI
- `egui/eframe` desktop UI in `src/app.rs`
- Adler-32 implementation backed by the `adler2` crate in `src/hash.rs`
- Theme handling with a fixed Material flat blue accent in `src/theme.rs`
- CLI argument parsing with `clap`:
  - `--text` to hash an inline string
  - `--file` to hash the raw bytes of a file
  - `--stdin` to hash bytes read from standard input
  - `--format` to choose hex, decimal, or both
- JetBrains Mono embedded directly in the executable
- No persistence: the app reads and writes no settings or data files

## Layout

```text
src/
  main.rs               CLI entry point and GUI launch
  app.rs                desktop UI
  hash.rs               Adler-32 calculation
  theme.rs              themes and the Material blue accent

assets/
  fonts/
    JetBrainsMono-Regular.ttf
    OFL.txt

packaging/
  windows/AdlerIt.iss

Scripts/                build and release helpers

quickstart.txt          user-facing setup and usage handout
```

## Command Line

Run the executable with any input option to use the command line; omit all of
them to launch the desktop app. Run `adlerit --help` for the full reference.

```shell
adlerit --text "Wikipedia"
# 11e60398

printf 'Wikipedia' | adlerit --stdin
adlerit --file ./payload.bin --format both
```

| Option | Description | Example |
|---|---|---|
| `--text <TEXT>` | Hash an inline string | `adlerit --text "Wikipedia"` |
| `--file <PATH>` | Hash the raw bytes of a file | `adlerit --file ./payload.bin` |
| `--stdin` | Hash bytes read from standard input | `printf 'Wikipedia' \| adlerit --stdin` |
| `--format <FORMAT>` | Output `hex` (default), `decimal`, or `both` | `adlerit --text hi --format both` |

The input options are mutually exclusive. With `--format both` the hex value is
printed first, then the decimal value, separated by two spaces.

## App Data

AdlerIt stores nothing. It writes no settings, history, cache, or data files,
and reads only the input you give it on the command line or in the window. There
is nothing to back up, clear, or migrate between versions.

## Security Features

AdlerIt has a deliberately small surface area:

- It performs no network access at all: no update checks, telemetry, or outbound
  connections.
- It only reads the input you provide — inline text, a named file, or standard
  input — and writes the checksum to standard output or the desktop window.
- It persists no state, so there is no settings or data file to tamper with.
- An offline-capability guard runs during the Windows build (ported from
  TypeText): it asserts the dependency graph contains no networking or registry
  crates and scans the binary for capability markers (URL opening, web requests,
  startup-registry writes, GitHub API). The build fails if any appear. A matching
  audit runs in CI on every pull request.
- The dependency tree is audited against the RustSec advisory database in CI and
  release, with a scheduled weekly re-audit.
- Release binaries are scanned with Microsoft Defender and carry GitHub
  build-provenance attestations.

Adler-32 is a fast checksum for integrity and error detection, not a
cryptographic hash. Do not use it to verify authenticity against a malicious
party or to store or compare secrets.

## Build And Run

AdlerIt currently targets Windows only. Release builds are produced natively on
Windows by the GitHub Actions workflow in `.github/workflows/release.yml`.

Release artifacts (each a single file):

```text
AdlerIt-Windows-x64.exe          portable
AdlerIt-Windows-x64-Setup.exe    installer
```

Choose the package that matches the environment:

- `AdlerIt-Windows-x64.exe` is the portable build — a single self-contained
  executable; download and run it, no installation required.
- `AdlerIt-Windows-x64-Setup.exe` installs AdlerIt and can add it to PATH.

Both are native 64-bit Windows applications with no bundled browser engine or web
application runtime.

To publish a GitHub Release, push a version tag in `vX.X.X` format:

```bash
git tag v0.1.0
git push origin v0.1.0
```

Before creating the tag, run **Build release apps** manually from the GitHub
Actions page and enter the intended `vX.X.X` tag. This dry run uses the real
release jobs to check formatting, tests, Clippy, the dependency audit, the
offline-capability guard, the portable and installer packages, and the Microsoft
Defender scan. It verifies the final artifact set and previews the release notes,
but it does not create a tag, provenance attestations, or a GitHub Release.

The `version` in `Cargo.toml` is the single source of truth for the AdlerIt
release version. Update it, let Cargo refresh `Cargo.lock`, commit both files,
then create a matching `vX.X.X` tag.

`Scripts/generate-release-notes.sh` generates the release changelog from commits
since the previous `vX.X.X` tag, plus a full diff link. GitHub displays a SHA-256
digest for each release asset, so separate `.sha256` files are not attached to
the release.

GitHub release artifacts also include build provenance attestations. Verify a
downloaded artifact with the GitHub CLI:

```bash
gh attestation verify AdlerIt-Windows-x64.exe --repo Joshndroid/AdlerIt
```

Attestations prove the artifact was produced by this repository's GitHub Actions
workflow.

### Release Security Checks

The release workflow must complete these checks before it publishes any GitHub
Release:

- The offline-capability guard confirms the build pulls in no networking or
  registry crates and that the binary contains no network, URL-opening, or
  startup-registry capability markers.
- Microsoft Defender scans the portable executable and the installer using
  current signatures. Any detection, unavailable Defender service, or incomplete
  scan blocks release publication.
- GitHub generates provenance attestations for the exact artifacts that passed
  those checks before they are attached to the release.

A successful scan means that the named security service reported no detections
at build time. It is an additional release safeguard, not a guarantee that
software can never contain or later develop a security issue.

Release verification also audits `Cargo.lock` against the current RustSec
advisory database. A separate scheduled workflow repeats that audit weekly so a
new advisory can block subsequent builds even when the lockfile has not changed.

Run the app and tests during development:

```bash
cargo run
cargo test
```

### Windows

Build the portable app (a single AdlerIt-Windows-x64.exe):

```powershell
Scripts\build-windows-portable.ps1
```

The Windows build targets 64-bit Windows:

```text
x86_64-pc-windows-msvc
```

If Rust says the target is missing, install it once:

```powershell
rustup target add x86_64-pc-windows-msvc
```

Build the installer:

```powershell
Scripts\build-windows-installer.ps1
```

The installer script requires Inno Setup 6. On GitHub Actions this is installed
with Chocolatey before the script runs. Both build scripts run the
offline-capability guard before packaging.

Outputs:

```text
dist\AdlerIt-Windows-x64.exe
dist\AdlerIt-Windows-x64.exe.sha256
dist\AdlerIt-Windows-x64-Setup.exe
dist\AdlerIt-Windows-x64-Setup.exe.sha256
```

## Appearance

The desktop app has a theme menu in the top-right corner with `System`, `Light`,
and `Dark` modes. The accent is a fixed Material flat blue (`#2196F3`). Because
AdlerIt persists nothing, the theme returns to `System` each launch; `System`
follows the operating system's current light/dark setting.

## Bundled Fonts

AdlerIt bundles JetBrains Mono Regular for a consistent UI font across platforms.
The font is licensed under the SIL Open Font License 1.1; see
`assets/fonts/OFL.txt`.

## License

AdlerIt is released under the MIT License; see `LICENSE`.
