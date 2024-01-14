
fix:
  cargo fmt
  git add -A
  git commit -m "fix: cargo fmt"
  cargo fix
  git add -A
  git commit -m "fix: cargo fix"
  cargo clippy --fix
  git add -A
  git commit -m "fix: cargo clippy --fix"