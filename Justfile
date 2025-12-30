# Justfile for PolyShark

# Run the project
run:
    cargo run

# Run with release optimizations
run-release:
    cargo run --release

# Format code
fmt:
    cargo fmt

# Check for errors
check:
    cargo check

# Lint code
clippy:
    cargo clippy

# Create documenation
doc:
    cargo doc --open
