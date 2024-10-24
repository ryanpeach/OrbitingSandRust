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
  cargo rustdoc

# Uses tracy to profile an example
[linux]
trace example:
  #!/usr/bin/env bash

  # Function to clean up the Tracy process
  cleanup() {
    if [ -n "$TRACY_PID" ] && ps -p $TRACY_PID > /dev/null; then
      echo "Killing Tracy process..."
      kill $TRACY_PID
      wait $TRACY_PID
    fi
  }

  # Set up trap to ensure cleanup runs on script exit, regardless of success or error
  trap cleanup EXIT

  # Start Tracy in the background and capture its PID
  capture-release -o my_capture.tracy &
  TRACY_PID=$!
  echo "Started Tracy with PID $TRACY_PID"

  # Run the cargo command with the necessary feature
  cargo run --features bevy/trace_tracy --example {{example}}

  # Run tracy on the captured file
  echo "Opening Tracy with the capture..."
  tracy my_capture.tracy

# Runs a benchmark, but this only works on linux
[linux]
bench:
    cargo bench

doc:
  cp -r assets/docs target/doc/assets
  RUSTDOCFLAGS="--html-in-header katex-header.html" cargo doc --no-deps --open
