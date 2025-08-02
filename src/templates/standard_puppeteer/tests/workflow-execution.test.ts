// Test for the refactored workflow execution
// Verifies that JSON workflow parsing and typed effect execution works correctly

import { createExecutor, WorkflowValidator } from '../lib/runtime';
import { workflow } from '../src/workflow';
import * as fs from 'fs';
import * as path from 'path';

// @ts-ignore
describe('Workflow Execution', () => {
  // @ts-ignore
  it('should validate and parse JSON workflow', () => {
    // Load the JSON workflow
    const jsonPath = path.join(__dirname, '../src/workflow.json');
    const jsonContent = fs.readFileSync(jsonPath, 'utf-8');
    const jsonWorkflow = JSON.parse(jsonContent);
    
    // Validate the JSON workflow
    const validator = new WorkflowValidator();
    const result = validator.validate(jsonWorkflow);
    
    if (!result.valid) {
      console.log('Validation errors:', result.errors);
    }
    // @ts-ignore
    expect(result.valid).toBe(true);
    // @ts-ignore
    expect(result.errors).toHaveLength(0);
  });
  
  // @ts-ignore
  it('should create executor from JSON workflow', () => {
    // This should not throw
    const executor = createExecutor(workflow);
    // @ts-ignore
    expect(executor).toBeDefined();
  });
  
  // @ts-ignore
  it('should reject invalid workflow JSON', () => {
    const invalidWorkflow = {
      workflow_id: 'test',
      // Missing required fields
    };
    
    // @ts-ignore
    expect(() => createExecutor(invalidWorkflow)).toThrow('Invalid workflow definition');
  });
  
  // @ts-ignore
  it('should validate effect dependencies', () => {
    const workflowWithBadDeps = {
      ...workflow,
      effects: [
        {
          type: 'navigate',
          navigate: {
            id: 'step1',
            dependencies: ['non_existent'],
            params: { url: 'https://example.com' }
          }
        }
      ]
    };
    
    // @ts-ignore
    expect(() => createExecutor(workflowWithBadDeps)).toThrow();
  });
  
  // @ts-ignore
  it('should validate circular dependencies', () => {
    const validator = new WorkflowValidator();
    const workflowWithCircularDeps = {
      ...workflow,
      effects: [
        {
          type: 'navigate',
          navigate: {
            id: 'step1',
            dependencies: ['step2'],
            params: { url: 'https://example.com' }
          }
        },
        {
          type: 'click',
          click: {
            id: 'step2',
            dependencies: ['step1'],
            params: { selector: '.button' }
          }
        }
      ]
    };
    
    const result = validator.validate(workflowWithCircularDeps);
    // @ts-ignore
    expect(result.valid).toBe(false);
    // @ts-ignore
    expect(result.errors.some(e => e.message.includes('Circular dependency'))).toBe(true);
  });
});