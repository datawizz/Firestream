#!/usr/bin/env node

/**
 * Generate TypeScript types from Supabase schema
 *
 * This script uses the Supabase CLI to generate TypeScript types
 * from the database schema. The types are output to stdout and can
 * be redirected to a file.
 *
 * Usage:
 *   node scripts/generate-types.js > src/types/database.ts
 *   pnpm db:generate-types
 *
 * Prerequisites:
 *   - Supabase CLI must be installed
 *   - Supabase project must be configured
 *   - Database migrations must be applied
 */

const { execSync } = require('child_process');
const path = require('path');

const AUTH_DIR = path.join(__dirname, '..');

try {
  console.error('Generating TypeScript types from Supabase schema...');

  // Run the Supabase CLI command to generate types
  const command = 'pnpm exec supabase gen types typescript --local';

  const output = execSync(command, {
    cwd: AUTH_DIR,
    encoding: 'utf8',
    stdio: ['inherit', 'pipe', 'inherit']
  });

  // Output the generated types to stdout
  console.log(output);

  console.error('TypeScript types generated successfully!');
} catch (error) {
  console.error('Error generating types:', error.message);
  process.exit(1);
}
