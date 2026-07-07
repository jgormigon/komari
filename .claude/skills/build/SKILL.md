---
name: build
description: How to build, compile-check, lint, and produce a runnable/testable executable for the komari project (a Rust workspace with a Dioxus desktop app in `ui/`, bot logic in `backend/`, and a `platforms` crate, linking OpenCV and ONNX runtime via vcpkg). Use this whenever building this project, running `cargo check`/`cargo test`/`cargo clippy`/`dx build` here, verifying that a code change compiles, or producing an app binary to test in-game. This machine has several environment-specific gotchas (protoc/libclang paths not on PATH, a Git-Bash-only linker bug, a test-execution permission quirk) that silently waste a lot of time if rediscovered from scratch each session — check this skill first.
---

# Building and testing komari

## Quick reference

Fast inner loop while editing `backend/` (seconds, no linking of a full binary):

```powershell
$env:PROTOC = "C:\Users\jacop\AppData\Local\Microsoft\WinGet\Packages\Google.Protobuf_Microsoft.Winget.Source_8wekyb3d8bbwe\bin\protoc.exe"
$env:LIBCLANG_PATH = "G:\Program Files\LLVM\bin"
cargo check -p backend --tests
cargo clippy -p backend --tests -- -D warnings
```

Full runnable app (needed to actually test bot behavior in-game):

```powershell
$env:PROTOC = "C:\Users\jacop\AppData\Local\Microsoft\WinGet\Packages\Google.Protobuf_Microsoft.Winget.Source_8wekyb3d8bbwe\bin\protoc.exe"
$env:LIBCLANG_PATH = "G:\Program Files\LLVM\bin"
dx build --package ui
# binary lands at: target\dx\ui\debug\windows\app\ui.exe
```

The rest of this file explains why each piece is needed and what to do when something doesn't match what's documented here.

## Always use PowerShell, never Git Bash

This is the single most important thing to get right on this machine. Git Bash (the default `Bash` tool) has a broken invocation of `link.exe` that fails with:

```
error: linking with `link.exe` failed: exit code: 1
...
link: missing operand after ' ■'
```

This happens on essentially any cargo command that needs to link a binary — `cargo build`, `cargo test`, `cargo clippy`, and `dx build`'s underlying cargo invocation. It is a shell/environment quirk, not caused by code changes — it reproduces identically on completely unmodified code (verified via `git stash`). `cargo check` can sometimes appear to work in Git Bash, but becomes unreliable the moment `build.rs` itself changes, since cargo then needs to relink the build-script binary and hits the same failure.

The fix is simply: always run cargo/dx commands through the **PowerShell** tool, never the **Bash** tool, on this machine. If a build/test/clippy command fails with the `■` linker error above, that's the tell — don't debug the code, just rerun the same command via PowerShell.

## Required environment variables

Some of these are expected to already be permanently configured in `%USERPROFILE%\.cargo\config.toml` (per the project README):

```toml
[env]
OPENCV_DISABLE_PROBES = "environment,pkg_config,cmake,vcpkg_cmake"
VCPKGRS_TRIPLET = "x64-windows-static"
VCPKG_ROOT = "G:/code/vcpkg"
```

If a build fails complaining about OpenCV/vcpkg discovery, check that these are actually present in that file — they should be a one-time setup, not something to redo per session.

Two more are needed but were **not** found in that config file as of this writing, so set them per PowerShell session (or see "making this permanent" below):

- **`PROTOC`** — the build depends on `tonic-build`/`prost-build`, which needs a real `protoc` binary. One is installed via winget (package `Google.Protobuf`) but isn't on `PATH`. Last found at:
  ```
  C:\Users\jacop\AppData\Local\Microsoft\WinGet\Packages\Google.Protobuf_Microsoft.Winget.Source_8wekyb3d8bbwe\bin\protoc.exe
  ```
  If that path no longer exists, rediscover it with:
  ```powershell
  Get-ChildItem -Recurse -Filter protoc.exe "$env:LOCALAPPDATA\Microsoft\WinGet\Packages" -ErrorAction SilentlyContinue
  ```
  The failure mode without this set is a build-script panic: `Could not find protoc...`.

- **`LIBCLANG_PATH`** — the `clang-sys` crate (used transitively for the OpenCV bindings) needs to find `libclang.dll`. LLVM is installed at `G:\Program Files\LLVM`, which has `libclang.dll` directly in its `bin` folder, so:
  ```
  LIBCLANG_PATH = "G:\Program Files\LLVM\bin"
  ```
  The failure mode without this set is a build-script panic mentioning `couldn't find any valid shared libraries matching: ['clang.dll', 'libclang.dll']`.

### Making these permanent

Both of these are static paths tied to this machine, not to a specific code change, so it's reasonable to add them to the same `%USERPROFILE%\.cargo\config.toml` shown above instead of setting them by hand every session:

```toml
[env]
OPENCV_DISABLE_PROBES = "environment,pkg_config,cmake,vcpkg_cmake"
VCPKGRS_TRIPLET = "x64-windows-static"
VCPKG_ROOT = "G:/code/vcpkg"
PROTOC = "C:/Users/jacop/AppData/Local/Microsoft/WinGet/Packages/Google.Protobuf_Microsoft.Winget.Source_8wekyb3d8bbwe/bin/protoc.exe"
LIBCLANG_PATH = "G:/Program Files/LLVM/bin"
```

This is a global, machine-wide file outside the repo, so ask the user before editing it — don't do it silently just because a build needs it.

## Fast inner loop: checking `backend` changes

Most day-to-day changes are in `backend/` (the bot logic). You very rarely need to rebuild the whole desktop app just to verify a change compiles or passes lints — `cargo check` doesn't need to link a full binary, so it's seconds instead of minutes:

```powershell
cargo check -p backend --tests
cargo clippy -p backend --tests -- -D warnings
```

Use these first, every time, before reaching for a full `dx build`.

### Running actual tests

`cargo test -p backend --lib <filter>` compiles and links the test binary fine, but then fails to *execute* it:

```
Caused by:
  The requested operation requires elevation. (os error 740)
```

This is an unresolved environment/permissions quirk on this machine (possibly a manifest requiring admin rights inherited from a GUI-related dependency pulled in by `backend`'s lib target), not something caused by code changes. Until it's resolved, `cargo check -p backend --tests` is the practical substitute — it verifies the test code compiles and type-checks (including `assert_matches!`, mock expectations, etc.) but does **not** actually run the assertions. Say so explicitly if reporting results based on `check` alone — it's not the same guarantee as green tests.

## Producing a runnable app to test in-game

Compiling `backend` only proves the logic type-checks — to actually see the bot do anything, you need the desktop app built and launched. This project uses the Dioxus CLI (`dx`, must be pre-installed):

```powershell
dx build --package ui                # debug, faster iteration
dx build --release --package ui      # release, per the README - use for real/distributed testing
```

A cold debug build takes on the order of a few minutes (first build in a given `target/` directory); incremental rebuilds after that are much faster, as long as you keep using PowerShell consistently (switching back to Git Bash mid-stream can leave the target directory in a state where the next PowerShell build has to redo work).

Output binary:

```
target\dx\ui\debug\windows\app\ui.exe        (debug build)
target\dx\ui\release\windows\app\ui.exe      (release build - same pattern, double check the exact path since it hasn't been directly confirmed)
```

`dx` may print a warning like `dx and dioxus versions are incompatible! dx version: 0.7.9, dioxus versions: [0.7.2]`. This has been benign — the build still succeeds. Don't treat it as a blocker on its own; only investigate further if the build actually fails.

## Working across git worktrees

This repo is frequently worked on via git worktrees, e.g. `.claude/worktrees/<branch-name>/`. Each worktree has its own independent `target/` directory — there is no shared build cache between them. This means the *first* build in a worktree you haven't built in before will always take the full cold-build time, regardless of how much has already been built elsewhere in the repo. That's expected, not a sign something is wrong.

Since binaries land under that worktree's own `target/`, when you tell the user where to find a build, give the path relative to the worktree you actually built in, not a generic repo-root guess.
