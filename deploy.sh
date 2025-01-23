#!/bin/bash

# Format the code
cargo fmt

# Build and compile the Rust program
cargo build --release

# Copy the executable to /usr/local/bin
sudo cp target/release/cnp /usr/local/bin/

# Add permissions to execute the program
sudo chmod +x /usr/local/bin/cnp

echo "Deployment complete!"