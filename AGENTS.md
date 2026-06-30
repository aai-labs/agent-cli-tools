# AGENTS.md

## Project context

> Loads on every agent invocation. Keep short.

- **Stack / versions** — Rust 2021 CLI; Tokio, Clap, Reqwest; Rust toolchain `<unpinned>`.
- **Package manager** — Cargo with the committed `Cargo.lock`.
- **Commands** — Install/build: `cargo build`; dev: `cargo run -- --help`; check: `cargo check --all-targets --all-features`; format: `cargo fmt --check`; lint: `cargo clippy --all-targets -- -D warnings`; test all: `scripts/run-tests.sh safe`; test single: `cargo test <test-name>`.
- **Non-obvious patterns** — Successful command output is JSON on stdout and errors are structured JSON on stderr; preserve this contract.
- **Non-obvious patterns** — Keep auth application centralized in `src/http.rs`; service modules under `src/services/` describe provider requests and response handling.
- **Non-obvious patterns** — List and search commands aggregate provider pagination up to `--limit` while preserving the provider response shape.
- **Non-obvious patterns** — Live E2E tests are ignored by default and require `AAI_E2E_CONFIG`; run them through `scripts/run-tests.sh live`.
- **Non-obvious patterns** — Keep credentials and generated local configs under ignored `local/`; never commit them.
- **Non-obvious patterns** — Bundled Agent Skills live under inert `bundled/skills/` package assets; provider command/service changes must update the matching skill and validation coverage when auth, flags, response shape, pagination, or workflows change.

## Agent work loop

1. **Plan** — For non-trivial work, explore the codebase and clarify material ambiguity before large edits.
2. **Outline** — Before substantive changes, state the intended files, behavior, and verification.
3. **Implement** — Follow the pointers below; stay within scope; do not refactor unrelated code.
4. **Verify** — Run focused tests while iterating and `scripts/run-tests.sh safe` before completion when practical.
5. **Document** — After user-facing changes, ensure `README.md`, command help, and `docs/` match current behavior.

Whenever working, consult:

- `CODE_GUIDELINES.md` — CLI/API boundaries, architecture heuristics, error handling, and review habits
- `TESTING.md` — Rust unit, integration, and ignored live E2E conventions

Those files use **MUST** / **SHOULD** / **MAY** where strictness matters.

## Guardrails

- **Always** — Follow MUST rules in guideline files; preserve public CLI and JSON contracts unless intentionally changing them; run safe diagnostics before asking the user.
- **Ask first** — Destructive or system-wide actions; installing system-wide dependencies; breaking CLI or output changes.
- **Never** — Commit credentials; silently deviate from an approved plan; revert unrelated user changes; invent extra scope mid-task.

## Coding core

Prefer simple, linear flows; composition over inheritance; immutable data where practical; and provider behavior grouped under its service module. Reuse shared request, auth, config, and output infrastructure instead of duplicating it.

## Maintaining guidelines

Record durable, repeatable conventions in the appropriate guideline file or `docs/`. Keep this file small and update its project context when commands or core patterns change.
