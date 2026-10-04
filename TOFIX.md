# TOFIX

Findings from a code scan on 2026-10-04.

## High

- `src/main.rs:1` - the whole crate is a `println!("Hello, World!")` stub, yet it has been released up to 0.1.7 (`Cargo.toml:3`) and is described as a pandoc implementation; either implement a first real conversion (e.g. markdown to HTML via `pulldown-cmark`) or stop cutting releases until it does something.

## Medium

- `README.md:2` - claims "A subset of pandoc implemented in Rust" with no usage, supported formats, or a note that nothing is implemented yet; state the current status and document the CLI once one exists.
- `src/main.rs:11` - the only test just calls `main()` as a placeholder; replace it with a test of real behaviour when the first feature lands.

## Low

- `src/main.rs:1` - the shared `build.rs` emits `GIT_SHA`, `GIT_DESCRIBE`, `BUILD_TIMESTAMP`, `RUSTC_SEMVER` etc. (`build.rs:89-95`) but nothing reads them; add the fleet's usual `--version` output (as the other rs* repos do) so the build script is not dead weight.
- `Cargo.toml:6` / `README.md:2` / `docs/src/introduction.md:3` - the description wording disagrees ("Rust version of pandoc" vs "A subset of pandoc implemented in Rust"); pick one and use it in all three, and add `keywords`/`categories` to `Cargo.toml` matching `config/project.lua`.
