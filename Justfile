
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