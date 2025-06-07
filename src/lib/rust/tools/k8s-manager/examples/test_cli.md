# Quick CLI Test

To test that the CLI is working correctly:

```bash
# Show help
cargo run -- --help

# Show cluster commands
cargo run -- cluster --help

# List clusters (safe, read-only operation)
cargo run -- cluster list

# Show version
cargo run -- --version
```

If all these commands work without errors, the CLI is properly set up!
