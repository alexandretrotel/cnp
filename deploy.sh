#!/bin/bash

# Build and compile the Rust program
cargo build --release

# Copy the executable to /usr/local/bin
sudo cp target/release/cnp /usr/local/bin/

echo "Deployment complete!"