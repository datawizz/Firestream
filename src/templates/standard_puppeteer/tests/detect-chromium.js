#!/usr/bin/env node

const { execSync } = require('child_process');
const fs = require('fs');

// Common chromium/chrome executable names and paths
const possiblePaths = [
  // Nix paths
  process.env.PUPPETEER_EXECUTABLE_PATH,
  '/nix/store/*/bin/chromium',
  '/nix/store/*/bin/chromium-browser',
  
  // System paths
  '/usr/bin/chromium',
  '/usr/bin/chromium-browser',
  '/usr/bin/google-chrome',
  '/usr/bin/google-chrome-stable',
  '/usr/local/bin/chromium',
  '/usr/local/bin/chromium-browser',
  
  // Alpine Linux
  '/usr/bin/chromium-browser',
  
  // Snap
  '/snap/bin/chromium',
  
  // Flatpak
  '/var/lib/flatpak/exports/bin/org.chromium.Chromium',
];

function findChromium() {
  // First check if explicitly set
  if (process.env.PUPPETEER_EXECUTABLE_PATH) {
    if (fs.existsSync(process.env.PUPPETEER_EXECUTABLE_PATH)) {
      return process.env.PUPPETEER_EXECUTABLE_PATH;
    }
  }
  
  // Try to find using which
  try {
    const chromiumPath = execSync('which chromium || which chromium-browser || which google-chrome || which google-chrome-stable', { encoding: 'utf8' }).trim();
    if (chromiumPath && fs.existsSync(chromiumPath)) {
      return chromiumPath;
    }
  } catch (e) {
    // which failed, continue with manual search
  }
  
  // Check common paths
  for (const path of possiblePaths) {
    if (path && path.includes('*')) {
      // Handle glob patterns (for nix store)
      try {
        const expanded = execSync(`ls -1 ${path} 2>/dev/null | head -1`, { encoding: 'utf8' }).trim();
        if (expanded && fs.existsSync(expanded)) {
          return expanded;
        }
      } catch (e) {
        // Glob expansion failed
      }
    } else if (path && fs.existsSync(path)) {
      return path;
    }
  }
  
  return null;
}

const chromiumPath = findChromium();

if (chromiumPath) {
  console.log(`Found Chromium at: ${chromiumPath}`);
  console.log('\nTo use it with Puppeteer, set:');
  console.log(`export PUPPETEER_EXECUTABLE_PATH="${chromiumPath}"`);
  console.log('export PUPPETEER_SKIP_CHROMIUM_DOWNLOAD=true');
} else {
  console.log('No Chromium installation found.');
  console.log('\nPuppeteer will download its own Chromium bundle.');
  console.log('To use system Chromium, install it with one of:');
  console.log('- apt-get install chromium-browser');
  console.log('- apk add chromium');
  console.log('- nix-shell (if using flake.nix)');
}

// Export for use in other scripts
module.exports = { findChromium };