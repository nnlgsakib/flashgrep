# Cross-Platform Validation Record

## Windows (local)

- Rust: `cargo` toolchain (local dev)
- Commit: current working tree

### Build/Test

- `cargo fmt --check`: pass
- `cargo test`: pass

### Parity checks

- Query match exit: pass (`0`)
- Query no-match exit: pass (`1`) using unique token
- Files/glob check (`src/*.[rl][si]`): pass
- Filesystem create/remove safety path: pass

### Notes

- Validation executed on `win32` local environment.

## Linux

- Status: pending manual execution
- Use `docs/cross-platform-manual-validation.md`

## macOS

- Status: pending manual execution
- Use `docs/cross-platform-manual-validation.md`
