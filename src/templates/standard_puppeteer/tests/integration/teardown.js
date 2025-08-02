const fs = require('fs').promises;
const path = require('path');

class TestTeardown {
  constructor(config = {}) {
    this.config = {
      preserveScreenshots: config.preserveScreenshots !== false,
      generateReport: config.generateReport !== false,
      ...config
    };
  }

  async cleanupTestArtifacts() {
    console.log('Cleaning up test artifacts...');
    
    // Clean temporary files but preserve screenshots and reports
    const tempDir = path.join(__dirname, '../results/temp');
    try {
      await fs.rm(tempDir, { recursive: true, force: true });
    } catch (error) {
      // Directory might not exist
    }
  }

  async generateTestReport(testResults) {
    if (!this.config.generateReport) {
      return;
    }

    console.log('Generating test report...');
    
    const reportPath = path.join(__dirname, '../results/reports', `test-report-${Date.now()}.json`);
    const htmlReportPath = reportPath.replace('.json', '.html');
    
    // Save JSON report
    await fs.writeFile(reportPath, JSON.stringify(testResults, null, 2));
    
    // Generate HTML report
    const htmlContent = this.generateHTMLReport(testResults);
    await fs.writeFile(htmlReportPath, htmlContent);
    
    console.log(`Test report saved to: ${htmlReportPath}`);
    return htmlReportPath;
  }

  generateHTMLReport(testResults) {
    const { passed, failed, duration, tests, screenshots } = testResults;
    const total = passed + failed;
    const passRate = total > 0 ? ((passed / total) * 100).toFixed(2) : 0;
    
    return `
<!DOCTYPE html>
<html>
<head>
  <title>Test Report - E-commerce Scraper</title>
  <style>
    body {
      font-family: Arial, sans-serif;
      margin: 20px;
      background-color: #f5f5f5;
    }
    .container {
      max-width: 1200px;
      margin: 0 auto;
      background: white;
      padding: 20px;
      border-radius: 8px;
      box-shadow: 0 2px 4px rgba(0,0,0,0.1);
    }
    h1 { color: #333; }
    .summary {
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
      gap: 20px;
      margin: 20px 0;
    }
    .stat {
      padding: 20px;
      background: #f8f9fa;
      border-radius: 8px;
      text-align: center;
    }
    .stat.passed { border-left: 4px solid #4caf50; }
    .stat.failed { border-left: 4px solid #f44336; }
    .stat.info { border-left: 4px solid #2196f3; }
    .test-list { margin-top: 30px; }
    .test {
      margin: 10px 0;
      padding: 15px;
      background: #f8f9fa;
      border-radius: 4px;
      border-left: 4px solid #ddd;
    }
    .test.passed { border-left-color: #4caf50; }
    .test.failed { border-left-color: #f44336; }
    .screenshot {
      margin: 10px 0;
      display: inline-block;
    }
    .screenshot img {
      max-width: 300px;
      border: 1px solid #ddd;
      border-radius: 4px;
    }
    .error {
      color: #f44336;
      margin-top: 10px;
      padding: 10px;
      background: #ffebee;
      border-radius: 4px;
    }
  </style>
</head>
<body>
  <div class="container">
    <h1>E-commerce Scraper Test Report</h1>
    <p>Generated: ${new Date().toLocaleString()}</p>
    
    <div class="summary">
      <div class="stat passed">
        <h3>${passed}</h3>
        <p>Passed</p>
      </div>
      <div class="stat failed">
        <h3>${failed}</h3>
        <p>Failed</p>
      </div>
      <div class="stat info">
        <h3>${passRate}%</h3>
        <p>Pass Rate</p>
      </div>
      <div class="stat info">
        <h3>${(duration / 1000).toFixed(2)}s</h3>
        <p>Duration</p>
      </div>
    </div>
    
    <div class="test-list">
      <h2>Test Results</h2>
      ${tests.map(test => `
        <div class="test ${test.status}">
          <h3>${test.name}</h3>
          <p>Duration: ${test.duration}ms</p>
          ${test.error ? `<div class="error">${test.error}</div>` : ''}
          ${test.screenshots && test.screenshots.length > 0 ? `
            <div class="screenshots">
              <h4>Screenshots:</h4>
              ${test.screenshots.map(screenshot => `
                <div class="screenshot">
                  <a href="${screenshot}" target="_blank">
                    <img src="${screenshot}" alt="${test.name}" />
                  </a>
                  <p>${path.basename(screenshot)}</p>
                </div>
              `).join('')}
            </div>
          ` : ''}
        </div>
      `).join('')}
    </div>
  </div>
</body>
</html>
    `;
  }

  async archiveResults() {
    if (!this.config.preserveScreenshots) {
      return;
    }

    const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
    const archiveDir = path.join(__dirname, '../results/archives', timestamp);
    
    try {
      await fs.mkdir(archiveDir, { recursive: true });
      
      // Copy screenshots from subdirectories
      const screenshotDir = path.join(__dirname, '../results/screenshots');
      let copiedCount = 0;
      
      try {
        const subdirs = await fs.readdir(screenshotDir);
        
        for (const subdir of subdirs) {
          const subdirPath = path.join(screenshotDir, subdir);
          const stat = await fs.stat(subdirPath);
          
          if (stat.isDirectory()) {
            // Create corresponding subdirectory in archive
            const archiveSubdir = path.join(archiveDir, subdir);
            await fs.mkdir(archiveSubdir, { recursive: true });
            
            // Copy all PNG files from this subdirectory
            const files = await fs.readdir(subdirPath);
            for (const file of files) {
              if (file.endsWith('.png')) {
                await fs.copyFile(
                  path.join(subdirPath, file),
                  path.join(archiveSubdir, file)
                );
                copiedCount++;
              }
            }
          }
        }
      } catch (error) {
        // Screenshots directory might not exist
        console.log('No screenshots to archive');
      }
      
      // Also copy reports if they exist
      const reportsDir = path.join(__dirname, '../results/reports');
      try {
        const reports = await fs.readdir(reportsDir);
        const reportsArchiveDir = path.join(archiveDir, 'reports');
        await fs.mkdir(reportsArchiveDir, { recursive: true });
        
        for (const report of reports) {
          if (report.endsWith('.json') || report.endsWith('.html')) {
            await fs.copyFile(
              path.join(reportsDir, report),
              path.join(reportsArchiveDir, report)
            );
          }
        }
      } catch (error) {
        // Reports directory might not exist
      }
      
      // Also copy data files if they exist
      const dataDir = path.join(__dirname, '../results/data');
      try {
        const dataFiles = await fs.readdir(dataDir);
        const dataArchiveDir = path.join(archiveDir, 'data');
        await fs.mkdir(dataArchiveDir, { recursive: true });
        
        for (const dataFile of dataFiles) {
          if (dataFile.endsWith('.json')) {
            await fs.copyFile(
              path.join(dataDir, dataFile),
              path.join(dataArchiveDir, dataFile)
            );
          }
        }
      } catch (error) {
        // Data directory might not exist
      }
      
      console.log(`Results archived to: ${archiveDir} (${copiedCount} screenshots)`);
      
      // Clean up old archives (keep only last 10)
      try {
        const archivesDir = path.join(__dirname, '../results/archives');
        const archives = await fs.readdir(archivesDir);
        const sortedArchives = archives.sort();
        
        if (sortedArchives.length > 10) {
          const toDelete = sortedArchives.slice(0, sortedArchives.length - 10);
          for (const archive of toDelete) {
            await fs.rm(path.join(archivesDir, archive), { recursive: true, force: true });
            console.log(`Removed old archive: ${archive}`);
          }
        }
      } catch (error) {
        // Ignore cleanup errors
      }
    } catch (error) {
      console.error('Failed to archive results:', error);
    }
  }

  async teardown(testResults) {
    try {
      // Generate report
      if (testResults) {
        await this.generateTestReport(testResults);
      }
      
      // Archive results
      await this.archiveResults();
      
      // Clean up temporary files
      await this.cleanupTestArtifacts();
      
      console.log('Teardown complete');
    } catch (error) {
      console.error('Teardown error:', error);
    }
  }
}

module.exports = TestTeardown;