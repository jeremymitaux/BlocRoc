# Contributing to BlocRoc

## Getting Started

1. Fork the repository and clone your fork.
2. Run `./scripts/setup.sh` to install all dependencies.
3. Create a feature branch: `git checkout -b feat/your-feature`.
4. Make your changes following the conventions in `CLAUDE.md`.
5. Run the relevant test suite (see below).
6. Open a pull request against `main`.

## Running Tests

```bash
# Pallet unit tests
cd roc-chain && cargo test

# Single pallet
cargo test -p pallet-ticket

# Frontend
cd roc-frontend && npm test

# Scanner app
cd roc-scanner && npm test

# Indexer
cd roc-indexer && npm test
```

## Code Review Checklist

Before opening a PR:

- [ ] All tests pass (`cargo test`, `npm test`)
- [ ] No `unwrap()` or `expect()` in production Rust paths
- [ ] Every new dispatchable has a `#[pallet::weight]` annotation
- [ ] Events are emitted for all state-changing dispatchables
- [ ] New storage items have `///` doc comments explaining invariants
- [ ] TypeScript has no `any` casts without a comment explaining why
- [ ] Commit messages follow Conventional Commits (see `CLAUDE.md`)

## Dependency Updates

Update Substrate/FRAME versions by bumping version strings in the root `Cargo.toml`
`[workspace.dependencies]` table only — never in individual crate `Cargo.toml` files.
