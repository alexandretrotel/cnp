#!/bin/bash

# Create temporary directory
TEMP_DIR=$(mktemp -d)
echo "Working in $TEMP_DIR"

# Create package.json
cat > "$TEMP_DIR/package.json" << EOL
{
  "name": "test-node-project",
  "version": "1.0.0",
  "description": "Test project for cnp",
  "dependencies": {
    "react": "^18.2.0",
    "@vercel/analytics": "^1.0.0",
    "lodash": "^4.17.21"
  },
  "devDependencies": {
    "eslint": "^8.0.0"
  }
}
EOL

# Function to clean node_modules and lock files
clean_dir() {
  rm -rf node_modules package-lock.json yarn.lock pnpm-lock.yaml bun.lock
}

# Generate npm lock file
cd "$TEMP_DIR"
echo "Generating package-lock.json..."
npm install
mv package-lock.json package-lock-test.json
clean_dir

# Generate yarn lock file
echo "Generating yarn.lock..."
yarn install
mv yarn.lock yarn-test.lock
clean_dir

# Generate pnpm lock file
echo "Generating pnpm-lock.yaml..."
pnpm install
mv pnpm-lock.yaml pnpm-lock-test.yaml
clean_dir

# Generate bun lock file
echo "Generating bun.lock..."
bun install
mv bun.lock bun-lock-test.lock
clean_dir

# Move lock files to project root
mv package-lock-test.json yarn-test.lock pnpm-lock-test.yaml bun-lock-test.lock "$OLDPWD"
echo "Lock files generated in $(pwd)"

# Clean up
cd "$OLDPWD"
rm -rf "$TEMP_DIR"

# Move lock files to test_fixtures directory
mv package-lock-test.json yarn-test.lock pnpm-lock-test.yaml bun-lock-test.lock test_fixtures/

# Print success message
echo "Lock files generated and moved to test_fixtures directory."
echo "All done!"