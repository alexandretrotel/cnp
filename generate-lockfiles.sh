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

# Create fixture JavaScript and TypeScript files
cat > "$TEMP_DIR/index.js" << EOL
import React from 'react';
import { analytics } from '@vercel/analytics';

// Example React component
function App() {
  return <div>Hello World</div>;
}
EOL

cat > "$TEMP_DIR/utils.ts" << EOL
import React from 'react';

// Utility function using React
export function renderComponent() {
  return React.createElement('div', null, 'Test');
}
EOL

cat > "$TEMP_DIR/aliased.js" << EOL
import { useState as useReactState } from 'react';
import { analytics as vercelAnalytics } from '@vercel/analytics';
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

# Move lock files and fixture files to project root
mv package-lock-test.json yarn-test.lock pnpm-lock-test.yaml bun-lock-test.lock index.js utils.ts aliased.js "$OLDPWD"
echo "Lock files and fixture files generated in $(pwd)"

# Clean up
cd "$OLDPWD"
rm -rf "$TEMP_DIR"

# Move lock files and fixture files to test_fixtures directory
mkdir -p test_fixtures
mv package-lock-test.json yarn-test.lock pnpm-lock-test.yaml bun-lock-test.lock index.js utils.ts aliased.js test_fixtures/

# Print success message
echo "Lock files and fixture files generated and moved to test_fixtures directory."
echo "All done!"