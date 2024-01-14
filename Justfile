# Run all fixers
fix:
  cargo fmt
  git add -A
  git commit -m "fix: cargo fmt" || true
  cargo fix
  git add -A
  git commit -m "fix: cargo fix" || true
  cargo clippy --fix
  git add -A
  git commit -m "fix: cargo clippy --fix" || true

# Build literally everything
build:
  cargo build --all-targets

check:
  cargo test
  cargo check --all-targets
  cargo clippy --all-targets