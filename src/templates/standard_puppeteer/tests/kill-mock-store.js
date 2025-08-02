#!/usr/bin/env node
const { execSync } = require('child_process');

try {
  // Try to find and kill processes using port 3000
  if (process.platform === 'darwin' || process.platform === 'linux') {
    try {
      // Try lsof first
      execSync('lsof -ti:3000 | xargs kill -9', { stdio: 'ignore' });
      console.log('Killed process on port 3000 using lsof');
    } catch (e) {
      // If lsof fails, try netstat
      try {
        const output = execSync('netstat -tlnp 2>/dev/null | grep :3000', { encoding: 'utf8' });
        const match = output.match(/(\d+)\/node/);
        if (match) {
          execSync(`kill -9 ${match[1]}`);
          console.log('Killed process on port 3000 using netstat');
        }
      } catch (e2) {
        // Try fuser as last resort
        try {
          execSync('fuser -k 3000/tcp', { stdio: 'ignore' });
          console.log('Killed process on port 3000 using fuser');
        } catch (e3) {
          console.log('Could not find process on port 3000');
        }
      }
    }
  } else if (process.platform === 'win32') {
    // Windows
    try {
      execSync('netstat -ano | findstr :3000', { encoding: 'utf8' }).split('\n').forEach(line => {
        const parts = line.trim().split(/\s+/);
        const pid = parts[parts.length - 1];
        if (pid && !isNaN(pid)) {
          execSync(`taskkill /F /PID ${pid}`, { stdio: 'ignore' });
        }
      });
      console.log('Killed process on port 3000');
    } catch (e) {
      console.log('Could not find process on port 3000');
    }
  }

  // Also try to kill any react-scripts processes
  try {
    if (process.platform === 'win32') {
      execSync('taskkill /F /IM node.exe /FI "WINDOWTITLE eq react-scripts*"', { stdio: 'ignore' });
    } else {
      execSync('pkill -f "react-scripts start"', { stdio: 'ignore' });
    }
    console.log('Killed react-scripts processes');
  } catch (e) {
    // Ignore
  }

  console.log('Mock store cleanup complete');
} catch (error) {
  console.error('Error during cleanup:', error.message);
}