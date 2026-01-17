// example_shop Scraper Entry Point
// Auto-generated from workflow definition

import * as dotenv from 'dotenv';
import { createExecutor } from '../lib';
import { workflow } from './workflow';
import { config, validateEnvironment } from './config';
import { logger } from '../lib/utils/logger';

// Load environment variables
dotenv.config();

async function main() {
  const startTime = Date.now();
  
  try {
    // Validate environment
    validateEnvironment();
    
    // Log startup
    logger.airflowLog('start', {
      workflow_id: workflow.workflow_id,
      config: {
        rate_limit: config.workflowConfig.rate_limit.requests_per_second,
        s3_bucket: config.workflowConfig.s3.bucket,
        effects_count: workflow.effects.length
      }
    });
    
    // Create and run executor
    const executor = createExecutor(workflow);
    const result = await executor.execute();
    
    // Calculate duration
    const duration = Date.now() - startTime;
    
    // Log completion for Airflow
    logger.airflowLog('complete', {
      workflow_id: workflow.workflow_id,
      duration_ms: duration,
      status: result.status,
      data_count: Object.keys(result.data).length,
      error_count: result.errors.length
    });
    
    // Exit with appropriate code
    if (result.status === 'completed') {
      if (result.errors.length > 0) {
        // Log errors but still exit 0 for partial success
        result.errors.forEach((error: any) => {
          logger.error(`Effect ${error.effect_id} failed: ${error.message}`, {
            effect_id: error.effect_id,
            error_type: error.error_type
          });
        });
      }
      process.exit(0);
    } else {
      // Failed - exit with error
      process.exit(1);
    }
    
  } catch (error) {
    const duration = Date.now() - startTime;
    
    logger.airflowLog('error', {
      workflow_id: workflow.workflow_id,
      duration_ms: duration,
      error: error instanceof Error ? error.message : 'Unknown error',
      stack: error instanceof Error ? error.stack : undefined
    });
    
    logger.error('Fatal error', error);
    process.exit(1);
  }
}

// Handle uncaught errors
process.on('uncaughtException', (error) => {
  logger.error('Uncaught exception', error);
  logger.airflowLog('error', {
    workflow_id: 'ecommerce_product_scraper',
    error: error.message,
    type: 'uncaught_exception'
  });
  process.exit(1);
});

process.on('unhandledRejection', (reason, promise) => {
  logger.error('Unhandled rejection', { reason, promise });
  logger.airflowLog('error', {
    workflow_id: 'ecommerce_product_scraper',
    error: reason instanceof Error ? reason.message : String(reason),
    type: 'unhandled_rejection'
  });
  process.exit(1);
});

// Handle graceful shutdown
process.on('SIGTERM', () => {
  logger.info('Received SIGTERM, shutting down gracefully');
  process.exit(0);
});

process.on('SIGINT', () => {
  logger.info('Received SIGINT, shutting down gracefully');
  process.exit(0);
});

// Run the scraper
if (require.main === module) {
  main();
}