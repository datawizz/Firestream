const { spawn } = require('child_process');
const path = require('path');
const fs = require('fs');

class TestSetup {
  constructor(config = {}) {
    this.config = {
      mockStorePort: config.mockStorePort || 3000,
      apiPort: config.apiPort || 3001,
      mockStorePath: config.mockStorePath || path.join(__dirname, './mock-store'),
      timeout: config.timeout || 60000,
      ...config
    };
    this.mockStoreProcess = null;
    this.apiServerProcess = null;
    this.isCleaningUp = false;
  }

  async startMockStore() {
    return new Promise((resolve, reject) => {
      console.log('Starting mock store...');
      
      // Change to mock store directory
      const cwd = this.config.mockStorePath;
      
      // Start the React development server
      this.mockStoreProcess = spawn('npm', ['start'], {
        cwd,
        env: {
          ...process.env,
          PORT: this.config.mockStorePort,
          BROWSER: 'none', // Don't open browser automatically
          CI: 'true' // Run in non-interactive mode
        }
      });

      let startupTimeout;
      
      // Handle stdout
      this.mockStoreProcess.stdout.on('data', (data) => {
        const output = data.toString();
        console.log(`Mock Store: ${output}`);
        
        // Check if port is already in use
        if (output.includes('Something is already running on port')) {
          clearTimeout(startupTimeout);
          reject(new Error(`Port ${this.config.mockStorePort} is already in use. Please stop any running processes on this port.`));
          return;
        }
        
        // Check if server is ready
        if (output.includes('compiled successfully') || output.includes('Compiled successfully')) {
          clearTimeout(startupTimeout);
          console.log(`Mock store is running on port ${this.config.mockStorePort}`);
          resolve();
        }
      });

      // Handle stderr
      this.mockStoreProcess.stderr.on('data', (data) => {
        console.error(`Mock Store Error: ${data}`);
      });

      // Handle process errors
      this.mockStoreProcess.on('error', (error) => {
        clearTimeout(startupTimeout);
        reject(new Error(`Failed to start mock store: ${error.message}`));
      });

      // Handle process exit
      this.mockStoreProcess.on('exit', (code) => {
        if (code !== 0 && code !== null) {
          clearTimeout(startupTimeout);
          reject(new Error(`Mock store process exited with code ${code}`));
        }
      });

      // Set timeout for startup
      startupTimeout = setTimeout(() => {
        this.stopMockStore();
        reject(new Error('Mock store startup timeout'));
      }, this.config.timeout);
    });
  }

  async startApiServer() {
    return new Promise((resolve, reject) => {
      console.log('Starting API server...');
      
      // Change to mock store directory
      const cwd = this.config.mockStorePath;
      
      // Start the Express API server
      this.apiServerProcess = spawn('node', ['server.js'], {
        cwd,
        env: {
          ...process.env,
          API_PORT: this.config.apiPort,
          PORT: this.config.apiPort
        }
      });

      let startupTimeout;
      
      // Handle stdout
      this.apiServerProcess.stdout.on('data', (data) => {
        const output = data.toString();
        console.log(`API Server: ${output}`);
        
        // Check if port is already in use
        if (output.includes('EADDRINUSE') || output.includes('address already in use')) {
          clearTimeout(startupTimeout);
          reject(new Error(`Port ${this.config.apiPort} is already in use. Please stop any running processes on this port.`));
          return;
        }
        
        // Check if server is ready
        if (output.includes(`running on http://localhost:${this.config.apiPort}`)) {
          clearTimeout(startupTimeout);
          console.log(`API server is running on port ${this.config.apiPort}`);
          resolve();
        }
      });

      // Handle stderr
      this.apiServerProcess.stderr.on('data', (data) => {
        console.error(`API Server Error: ${data}`);
      });

      // Handle process errors
      this.apiServerProcess.on('error', (error) => {
        clearTimeout(startupTimeout);
        reject(new Error(`Failed to start API server: ${error.message}`));
      });

      // Handle process exit
      this.apiServerProcess.on('exit', (code) => {
        if (code !== 0 && code !== null) {
          clearTimeout(startupTimeout);
          reject(new Error(`API server process exited with code ${code}`));
        }
      });

      // Set timeout for startup
      startupTimeout = setTimeout(() => {
        this.stopApiServer();
        reject(new Error('API server startup timeout'));
      }, this.config.timeout);
    });
  }

  async stopMockStore() {
    if (this.isCleaningUp || !this.mockStoreProcess) {
      return;
    }
    
    try {
      console.log('Stopping mock store...');
      this.mockStoreProcess.kill('SIGTERM');
      
      // Give it time to shutdown gracefully
      await new Promise(resolve => setTimeout(resolve, 2000));
      
      // Force kill if still running
      if (this.mockStoreProcess && !this.mockStoreProcess.killed) {
        this.mockStoreProcess.kill('SIGKILL');
      }
    } finally {
      this.mockStoreProcess = null;
    }
  }

  async stopApiServer() {
    if (this.isCleaningUp || !this.apiServerProcess) {
      return;
    }
    
    try {
      console.log('Stopping API server...');
      this.apiServerProcess.kill('SIGTERM');
      
      // Give it time to shutdown gracefully
      await new Promise(resolve => setTimeout(resolve, 2000));
      
      // Force kill if still running
      if (this.apiServerProcess && !this.apiServerProcess.killed) {
        this.apiServerProcess.kill('SIGKILL');
      }
    } finally {
      this.apiServerProcess = null;
    }
  }

  async ensureCleanState() {
    // Clear any session storage or cookies
    console.log('Ensuring clean test state...');
    
    // Create results directories if they don't exist
    const screenshotDir = path.join(__dirname, '../results/screenshots');
    const reportDir = path.join(__dirname, '../results/reports');
    
    [screenshotDir, reportDir].forEach(dir => {
      if (!fs.existsSync(dir)) {
        fs.mkdirSync(dir, { recursive: true });
      }
    });
  }

  async checkIfMockStoreIsRunning() {
    const axios = require('axios');
    try {
      const response = await axios.get(`http://localhost:${this.config.mockStorePort}`, {
        timeout: 2000,
        validateStatus: () => true
      });
      return response.status === 200;
    } catch (error) {
      return false;
    }
  }

  async checkIfApiServerIsRunning() {
    const axios = require('axios');
    try {
      const response = await axios.get(`http://localhost:${this.config.apiPort}/api/health`, {
        timeout: 2000,
        validateStatus: () => true
      });
      return response.status === 200 && response.data.status === 'ok';
    } catch (error) {
      return false;
    }
  }

  async setup() {
    try {
      await this.ensureCleanState();
      
      // Check if mock store is already running
      const isMockStoreRunning = await this.checkIfMockStoreIsRunning();
      if (isMockStoreRunning) {
        console.log(`Mock store is already running on port ${this.config.mockStorePort}`);
      } else {
        await this.startMockStore();
        // Wait a bit for server to be fully ready
        await new Promise(resolve => setTimeout(resolve, 3000));
      }
      
      // Check if API server is already running
      const isApiServerRunning = await this.checkIfApiServerIsRunning();
      if (isApiServerRunning) {
        console.log(`API server is already running on port ${this.config.apiPort}`);
      } else {
        await this.startApiServer();
        // Wait a bit for server to be fully ready
        await new Promise(resolve => setTimeout(resolve, 2000));
      }
      
      console.log('Both servers are ready!');
      
      return {
        mockStoreUrl: `http://localhost:${this.config.mockStorePort}`,
        apiUrl: `http://localhost:${this.config.apiPort}`,
        cleanup: () => this.teardown()
      };
    } catch (error) {
      await this.teardown();
      throw error;
    }
  }

  async teardown() {
    this.isCleaningUp = true;
    console.log('Tearing down test environment...');
    
    // Stop both servers in parallel
    await Promise.allSettled([
      this.stopMockStore(),
      this.stopApiServer()
    ]);
    
    this.isCleaningUp = false;
  }
}

module.exports = TestSetup;