use serde_json::json;
use std::fs;
use std::path::Path;
use tera::{Context, Tera};

/// Example usage of the Spark templatizer
#[allow(dead_code)]
pub fn example_usage() -> Result<(), Box<dyn std::error::Error>> {
    // Example 1: Generate a Scala Spark application
    generate_scala_app()?;

    // Example 2: Generate a Python Spark application
    generate_python_app()?;

    // Example 3: Generate both from the same configuration
    generate_from_shared_config()?;

    Ok(())
}

/// Generate a Scala Spark application
#[allow(dead_code)]
fn generate_scala_app() -> Result<(), Box<dyn std::error::Error>> {
    println!("Generating Scala Spark application...");

    // Get the project root directory
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let template_path =
        Path::new(&manifest_dir).join("src/templates/spark_scala_boilerplate/**/*.tera");

    // Load Scala templates
    let tera = Tera::new(template_path.to_str().unwrap())?;

    // Create context with Scala-specific configuration
    let context = Context::from_serialize(json!({
        "app_name": "Sales Analytics Pipeline",
        "organization": "com.example",
        "version": "1.0.0",
        "description": "Spark application for sales data analytics",
        "package_name": "com.example.analytics.sales",
        "main_class_name": "SalesAnalytics",

        // Scala specific
        "scala_version": "2.13.15",
        "spark_version": "3.5.1",
        "sbt_version": "1.1.7",

        // Features
        "s3_enabled": true,
        "delta_enabled": true,
        "config_enabled": true,
        "streaming_enabled": false,
        "mllib_enabled": false,

        // Data processing
        "input_format": "parquet",
        "output_format": "delta",
        "cache_input": true,

        // Configuration fields
        "config_fields": [
            {
                "name": "inputPath",
                "type": "String",
                "config_type": "String",
                "config_path": "input.path",
                "default_value": "s3a://data-lake/sales/raw",
                "env_var": "INPUT_PATH",
                "arg_index": 0
            },
            {
                "name": "outputPath",
                "type": "String",
                "config_type": "String",
                "config_path": "output.path",
                "default_value": "s3a://data-lake/sales/processed",
                "env_var": "OUTPUT_PATH",
                "arg_index": 1
            }
        ],

        // Transformations
        "transformations": [
            {
                "method": "filter",
                "args": ["col(\"amount\") > 0"]
            },
            {
                "method": "withColumn",
                "args": ["\"year\"", "year(col(\"date\"))"]
            }
        ],

        // Aggregations
        "aggregations": [
            {
                "function": "sum",
                "column": "amount",
                "alias": "total_sales"
            },
            {
                "function": "count",
                "column": "*",
                "alias": "transaction_count"
            }
        ],
        "group_by_columns": ["region", "year"],

        // Docker and K8s
        "docker_registry": "myregistry.io",
        "namespace": "analytics",
        "driver_memory": "4g",
        "executor_memory": "8g",
        "executor_instances": 5
    }))?;

    // Render specific files
    let build_sbt = tera.render("build.sbt.tera", &context)?;
    println!("Generated build.sbt:\n{}", &build_sbt[..200]); // Show first 200 chars

    Ok(())
}

/// Generate a Python Spark application
#[allow(dead_code)]
fn generate_python_app() -> Result<(), Box<dyn std::error::Error>> {
    println!("\nGenerating Python Spark application...");

    // Get the project root directory
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let template_path =
        Path::new(&manifest_dir).join("src/templates/spark_python_boilerplate/**/*.tera");

    // Load Python templates
    let tera = Tera::new(template_path.to_str().unwrap())?;

    // Create context with Python-specific configuration
    let context = Context::from_serialize(json!({
        "app_name": "Customer Segmentation",
        "organization": "DataScience Inc",
        "version": "2.0.0",
        "description": "PySpark application for customer segmentation using ML",
        "package_name": "customer_segmentation",
        "main_class_name": "CustomerSegmentation",

        // Python specific
        "python_version": "3.10",
        "python_major_version": "3",
        "pyspark_version": "3.5.1",

        // Features
        "s3_enabled": true,
        "delta_enabled": false,
        "config_enabled": true,
        "include_utils": true,
        "use_pre_commit": true,

        // Data processing
        "input_format": "csv",
        "output_format": "parquet",
        "config_format": "yaml",

        // Configuration fields
        "config_fields": [
            {
                "name": "input_path",
                "python_type": "str",
                "default_value": "s3a://ml-data/customers/raw",
                "env_var": "INPUT_PATH",
                "arg_index": 0
            },
            {
                "name": "model_path",
                "python_type": "str",
                "default_value": "s3a://ml-data/models/segmentation",
                "env_var": "MODEL_PATH"
            },
            {
                "name": "num_clusters",
                "python_type": "int",
                "python_default": "5",
                "default_value": "5",
                "env_var": "NUM_CLUSTERS"
            }
        ],

        // Custom transformation code
        "custom_transformation_code":
            "# Apply ML transformations\nfrom pyspark.ml.feature import VectorAssembler\nassembler = VectorAssembler(inputCols=['age', 'income', 'score'], outputCol='features')\ndf = assembler.transform(df)",

        // Testing
        "test_sample_data": [
            "('cust1', 25, 50000.0, 720)",
            "('cust2', 35, 75000.0, 680)",
            "('cust3', 45, 100000.0, 750)"
        ],
        "test_columns": ["customer_id", "age", "income", "credit_score"],

        // Docker and K8s
        "docker_registry": "ml-registry.io",
        "namespace": "ml-apps",
        "driver_memory": "4g",
        "executor_memory": "6g",
        "executor_instances": 3
    }))?;

    // Render specific files
    let main_py = tera.render("src/main.py.tera", &context)?;
    println!("Generated main.py:\n{}", &main_py[..200]); // Show first 200 chars

    Ok(())
}

/// Generate both applications from shared configuration
#[allow(dead_code)]
fn generate_from_shared_config() -> Result<(), Box<dyn std::error::Error>> {
    println!("\nGenerating both applications from shared config...");

    // Shared configuration
    let shared_config = json!({
        "app_name": "Data Processing Pipeline",
        "organization": "TechCorp",
        "version": "1.0.0",
        "description": "Multi-language Spark data processing pipeline",

        // Shared features
        "s3_enabled": true,
        "delta_enabled": true,
        "config_enabled": true,
        "monitoring_enabled": true,

        // Shared K8s/Docker config
        "docker_registry": "techcorp.io",
        "namespace": "data-processing",
        "service_account": "spark",
        "driver_memory": "2g",
        "executor_memory": "4g",
        "executor_instances": 3,

        // Shared Spark config
        "spark_configs": [
            {
                "key": "spark.sql.adaptive.enabled",
                "value": "true"
            },
            {
                "key": "spark.sql.adaptive.coalescePartitions.enabled",
                "value": "true"
            }
        ]
    });

    // Generate Scala version
    {
        let mut scala_config = shared_config.clone();
        scala_config["package_name"] = json!("com.techcorp.pipeline");
        scala_config["main_class_name"] = json!("DataPipeline");
        scala_config["scala_version"] = json!("2.13.15");

        let _context = Context::from_serialize(scala_config)?;
        // Render Scala templates...
        println!("Generated Scala version with package: com.techcorp.pipeline");
    }

    // Generate Python version
    {
        let mut python_config = shared_config.clone();
        python_config["package_name"] = json!("data_pipeline");
        python_config["main_class_name"] = json!("DataPipeline");
        python_config["python_version"] = json!("3.10");

        let _context = Context::from_serialize(python_config)?;
        // Render Python templates...
        println!("Generated Python version with package: data_pipeline");
    }

    println!("\nBoth applications generated with compatible configurations!");

    Ok(())
}

/// Helper function to render all templates to a directory
pub fn render_templates_to_directory(
    template_type: &str,
    context: Context,
    output_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    // Get the project root directory
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| {
        eprintln!("CARGO_MANIFEST_DIR not set, using current directory");
        ".".to_string()
    });

    // Determine template path - need to construct glob pattern as string
    let template_glob = match template_type {
        "scala" => "src/templates/spark_scala_boilerplate/**/*.tera",
        "python" => "src/templates/spark_python_boilerplate/**/*.tera",
        _ => return Err("Invalid template type. Use 'scala' or 'python'".into()),
    };

    // Construct the full glob pattern as a string (not using Path::join which escapes wildcards)
    let template_path_str = if manifest_dir == "." {
        template_glob.to_string()
    } else {
        format!("{}/{}", manifest_dir.trim_end_matches('/'), template_glob)
    };

    eprintln!("Loading templates from: {}", template_path_str);

    // Load templates
    let tera = match Tera::new(&template_path_str) {
        Ok(tera) => {
            eprintln!(
                "Successfully loaded {} templates",
                tera.get_template_names().count()
            );
            tera
        }
        Err(e) => {
            eprintln!("Failed to load templates: {}", e);
            return Err(Box::new(e));
        }
    };

    // Create output directory
    fs::create_dir_all(output_dir)?;

    // Render each template
    for template_name in tera.get_template_names() {
        // Skip non-.tera files
        if !template_name.ends_with(".tera") {
            continue;
        }

        // Render template
        let rendered = match tera.render(template_name, &context) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Failed to render template {}: {}", template_name, e);
                return Err(Box::new(e));
            }
        };

        // Determine output path (remove .tera extension and base directory)
        let output_file = template_name.trim_end_matches(".tera");

        // Remove the template type prefix from the path
        let relative_path = match template_type {
            "scala" => output_file
                .strip_prefix("spark_scala_boilerplate/")
                .unwrap_or(output_file),
            "python" => output_file
                .strip_prefix("spark_python_boilerplate/")
                .unwrap_or(output_file),
            _ => output_file,
        };

        let output_path = output_dir.join(relative_path);

        // Create parent directories if needed
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Write file
        fs::write(&output_path, rendered)?;
        println!("Generated: {}", output_path.display());
    }

    println!(
        "\nSuccessfully generated {} Spark application in: {}",
        template_type,
        output_dir.display()
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_render_scala_templates() {
        let temp_dir = TempDir::new().unwrap();

        // Load the example context from the template directory
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
        let example_context_path = Path::new(&manifest_dir)
            .join("src/templates/spark_scala_boilerplate/example_context.json");

        let context_json = fs::read_to_string(&example_context_path)
            .expect("Failed to read Scala example context");
        let context_value: serde_json::Value =
            serde_json::from_str(&context_json).expect("Failed to parse Scala example context");
        let context = Context::from_serialize(context_value).unwrap();

        let result = render_templates_to_directory("scala", context, temp_dir.path());

        if let Err(e) = &result {
            eprintln!("Failed to render Scala templates: {}", e);
        }
        assert!(result.is_ok());

        // Check if templates were actually rendered
        let build_sbt = temp_dir.path().join("build.sbt");
        if !build_sbt.exists() {
            eprintln!("build.sbt not found at: {}", build_sbt.display());
            eprintln!("Directory contents:");
            if let Ok(entries) = std::fs::read_dir(temp_dir.path()) {
                for entry in entries {
                    if let Ok(entry) = entry {
                        eprintln!("  - {}", entry.path().display());
                    }
                }
            }
        }
        assert!(build_sbt.exists());
    }

    #[test]
    fn test_render_python_templates() {
        let temp_dir = TempDir::new().unwrap();

        // Load the example context from the template directory
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
        let example_context_path = Path::new(&manifest_dir)
            .join("src/templates/spark_python_boilerplate/example_context.json");

        let context_json = fs::read_to_string(&example_context_path)
            .expect("Failed to read Python example context");
        let context_value: serde_json::Value =
            serde_json::from_str(&context_json).expect("Failed to parse Python example context");
        let context = Context::from_serialize(context_value).unwrap();

        let result = render_templates_to_directory("python", context, temp_dir.path());

        if let Err(e) = &result {
            eprintln!("Failed to render Python templates: {}", e);
        }
        assert!(result.is_ok());

        // Check if templates were actually rendered
        let setup_py = temp_dir.path().join("setup.py");
        if !setup_py.exists() {
            eprintln!("setup.py not found at: {}", setup_py.display());
            eprintln!("Directory contents:");
            if let Ok(entries) = std::fs::read_dir(temp_dir.path()) {
                for entry in entries {
                    if let Ok(entry) = entry {
                        eprintln!("  - {}", entry.path().display());
                    }
                }
            }
        }
        assert!(setup_py.exists());
    }
}
