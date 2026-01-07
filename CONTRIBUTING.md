# Contributing

Keep it simple, fast, and safe.

### Guidelines
- **Format**: Run `cargo fmt` before committing.
- **Lint**: Ensure `cargo clippy -- -D warnings` passes (no `allow` rules).
- **Test**: Add unit tests for logic and integration tests for CLI features.
- **Performance**: Avoid spawning sub-processes; use library APIs (like `gix`) where possible.
- **VCS**: Support both Git and Jujutsu.

### Workflow
1. Fork and branch.
2. Implement feature/fix.
3. Run `cargo test`.
4. Submit PR.
