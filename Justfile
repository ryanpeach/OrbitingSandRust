
fix:
  cargo fix
  git add -A
  git commit -m "fix: cargo fix"
  cargo clippy --fix
  git add -A
  git commit -m "fix: cargo clippy --fix"