const fs = require('fs');
const path = require('path');

class DataValidator {
  constructor() {
    this.validationResults = [];
  }

  validateProduct(product) {
    const errors = [];
    const warnings = [];
    
    // Required fields
    if (!product.id) {
      errors.push('Missing required field: id');
    } else if (typeof product.id !== 'string') {
      errors.push('Invalid type for id: expected string');
    } else if (!/^PROD-\d{4}$/.test(product.id)) {
      warnings.push(`Invalid product ID format: ${product.id}`);
    }
    
    if (!product.name) {
      errors.push('Missing required field: name');
    } else if (typeof product.name !== 'string') {
      errors.push('Invalid type for name: expected string');
    } else if (product.name.trim().length === 0) {
      errors.push('Product name cannot be empty');
    }
    
    // Optional fields with type validation
    if (product.price !== undefined) {
      if (typeof product.price !== 'number') {
        errors.push('Invalid type for price: expected number');
      } else if (product.price < 0) {
        errors.push('Price cannot be negative');
      } else if (!isFinite(product.price)) {
        errors.push('Price must be a finite number');
      }
    }
    
    if (product.inStock !== undefined) {
      if (typeof product.inStock !== 'boolean') {
        errors.push('Invalid type for inStock: expected boolean');
      }
    }
    
    return {
      valid: errors.length === 0,
      errors,
      warnings
    };
  }

  validateProductList(products) {
    if (!Array.isArray(products)) {
      return {
        valid: false,
        errors: ['Products must be an array'],
        warnings: []
      };
    }
    
    const results = {
      total: products.length,
      valid: 0,
      invalid: 0,
      errors: [],
      warnings: [],
      invalidProducts: []
    };
    
    products.forEach((product, index) => {
      const validation = this.validateProduct(product);
      
      if (validation.valid) {
        results.valid++;
      } else {
        results.invalid++;
        results.invalidProducts.push({
          index,
          product,
          errors: validation.errors
        });
      }
      
      results.errors.push(...validation.errors);
      results.warnings.push(...validation.warnings);
    });
    
    return results;
  }

  validateScrapedData(data) {
    const validation = {
      timestamp: new Date().toISOString(),
      results: {}
    };
    
    // Check if data exists
    if (!data) {
      validation.results.data = {
        valid: false,
        error: 'No data provided'
      };
      return validation;
    }
    
    // Validate metadata if present
    if (data.metadata) {
      validation.results.metadata = this.validateMetadata(data.metadata);
    }
    
    // Validate products
    if (data.products) {
      validation.results.products = this.validateProductList(data.products);
    } else if (Array.isArray(data)) {
      // Handle case where data is just an array of products
      validation.results.products = this.validateProductList(data);
    }
    
    // Calculate overall validity
    validation.valid = Object.values(validation.results).every(r => r.valid !== false);
    
    this.validationResults.push(validation);
    return validation;
  }

  validateMetadata(metadata) {
    const expected = {
      workflow_id: 'string',
      execution_id: 'string',
      timestamp: 'string',
      items_count: 'number'
    };
    
    const errors = [];
    
    Object.entries(expected).forEach(([key, type]) => {
      if (metadata[key] === undefined) {
        errors.push(`Missing metadata field: ${key}`);
      } else if (typeof metadata[key] !== type) {
        errors.push(`Invalid type for metadata.${key}: expected ${type}`);
      }
    });
    
    return {
      valid: errors.length === 0,
      errors
    };
  }

  compareWithExpected(actual, expected) {
    const comparison = {
      matches: true,
      differences: []
    };
    
    // Compare product counts
    if (actual.length !== expected.length) {
      comparison.matches = false;
      comparison.differences.push({
        type: 'count',
        expected: expected.length,
        actual: actual.length
      });
    }
    
    // Compare individual products
    const actualMap = new Map(actual.map(p => [p.id, p]));
    const expectedMap = new Map(expected.map(p => [p.id, p]));
    
    // Check for missing products
    expected.forEach(expectedProduct => {
      if (!actualMap.has(expectedProduct.id)) {
        comparison.matches = false;
        comparison.differences.push({
          type: 'missing',
          product: expectedProduct
        });
      }
    });
    
    // Check for extra products
    actual.forEach(actualProduct => {
      if (!expectedMap.has(actualProduct.id)) {
        comparison.matches = false;
        comparison.differences.push({
          type: 'extra',
          product: actualProduct
        });
      }
    });
    
    // Compare matching products
    actualMap.forEach((actualProduct, id) => {
      const expectedProduct = expectedMap.get(id);
      if (expectedProduct) {
        const fieldDifferences = this.compareProducts(actualProduct, expectedProduct);
        if (fieldDifferences.length > 0) {
          comparison.matches = false;
          comparison.differences.push({
            type: 'fields',
            id,
            differences: fieldDifferences
          });
        }
      }
    });
    
    return comparison;
  }

  compareProducts(actual, expected) {
    const differences = [];
    const fields = ['id', 'name', 'price', 'inStock'];
    
    fields.forEach(field => {
      if (actual[field] !== expected[field]) {
        differences.push({
          field,
          expected: expected[field],
          actual: actual[field]
        });
      }
    });
    
    return differences;
  }

  generateValidationReport(outputPath) {
    const report = {
      timestamp: new Date().toISOString(),
      summary: {
        totalValidations: this.validationResults.length,
        passed: this.validationResults.filter(r => r.valid).length,
        failed: this.validationResults.filter(r => !r.valid).length
      },
      details: this.validationResults
    };
    
    const reportPath = outputPath || path.join(__dirname, '../../results/reports', `validation-report-${Date.now()}.json`);
    fs.writeFileSync(reportPath, JSON.stringify(report, null, 2));
    
    console.log(`Validation report saved to: ${reportPath}`);
    return report;
  }

  validatePaginationSequence(pagesData) {
    const validation = {
      valid: true,
      errors: [],
      warnings: []
    };
    
    // Check for duplicate products across pages
    const allProductIds = new Set();
    let duplicates = [];
    
    pagesData.forEach((pageData, pageIndex) => {
      pageData.products.forEach(product => {
        if (allProductIds.has(product.id)) {
          duplicates.push({
            id: product.id,
            page: pageIndex + 1
          });
        }
        allProductIds.add(product.id);
      });
    });
    
    if (duplicates.length > 0) {
      validation.valid = false;
      validation.errors.push(`Found ${duplicates.length} duplicate products across pages`);
      validation.duplicates = duplicates;
    }
    
    // Verify sequential pagination
    pagesData.forEach((pageData, index) => {
      if (pageData.page !== index + 1) {
        validation.warnings.push(`Page number mismatch: expected ${index + 1}, got ${pageData.page}`);
      }
    });
    
    return validation;
  }
}

module.exports = DataValidator;