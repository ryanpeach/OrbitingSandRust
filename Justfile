# Run all fixers
fix:
  cargo fix
  git add -A
  git commit -m "fix: cargo fix" --no-verify || true
  cargo clippy --fix
  git add -A
  git commit -m "fix: cargo clippy --fix" --no-verify || true
  cargo fmt
  git add -A
  git commit -m "fix: cargo fmt" --no-verify || true

# Build literally everything
build:
  cargo build --all-targets

check:
  cargo test
  cargo check --all-targets
  cargo clippy --all-targets

doc:
  cp -r assets/docs target/doc/assets
  RUSTDOCFLAGS="--html-in-header katex-header.html" cargo doc --no-deps --open