#!/usr/bin/env node

const { execSync } = require('child_process');
const { glob } = require('glob');
const path = require('path');
const { mkdirpSync } = require('mkdirp');
const { rimrafSync } = require('rimraf');

const PROJECT_ROOT = path.resolve(__dirname, '../../../..');
const PROTO_DIR = path.join(PROJECT_ROOT, 'lib/proto');
const OUT_DIR = path.join(__dirname, '..', 'src', 'generated');

async function main() {
  console.log('Generating TypeScript types from Protocol Buffers...');
  console.log(`Proto directory: ${PROTO_DIR}`);
  console.log(`Output directory: ${OUT_DIR}`);

  // Clean and create output directory
  rimrafSync(OUT_DIR);
  mkdirpSync(OUT_DIR);

  // Find all proto files
  const protoFiles = await glob('**/*.proto', {
    cwd: PROTO_DIR,
    absolute: false,
  });

  // Filter out test files
  const filteredProtos = protoFiles.filter(file => {
    if (file.includes('test') || file.includes('/google/')) {
      console.log(`Skipping: ${file}`);
      return false;
    }
    return true;
  });

  console.log(`Found ${filteredProtos.length} proto files to process`);

  if (filteredProtos.length === 0) {
    console.log('No proto files found to process');
    return;
  }

  // Generate TypeScript files
  const protocArgs = [
    'protoc',
    `--proto_path=${PROTO_DIR}`,
    '--plugin=protoc-gen-es=node_modules/.bin/protoc-gen-es',
    `--es_out=${OUT_DIR}`,
    '--es_opt=target=ts',
    ...filteredProtos.map(f => path.join(PROTO_DIR, f))
  ];

  try {
    execSync(protocArgs.join(' '), {
      stdio: 'inherit',
      cwd: path.join(__dirname, '..'),
    });
    console.log('Proto generation completed successfully');
  } catch (error) {
    console.error('Proto generation failed:', error.message);
    process.exit(1);
  }
}

main().catch(error => {
  console.error('Error generating protos:', error);
  process.exit(1);
});
