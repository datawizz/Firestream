//! Utility functions and helpers for Superset dashboard generation

use serde_json::{json, Value};
use uuid::Uuid;

/// Generate a deterministic UUID from a string (for consistent IDs)
pub fn uuid_from_string(s: &str) -> Uuid {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    let hash = hasher.finish();

    // Create UUID from hash bytes
    let bytes = hash.to_le_bytes();
    let mut uuid_bytes = [0u8; 16];
    uuid_bytes[..8].copy_from_slice(&bytes);
    uuid_bytes[8..].copy_from_slice(&bytes);

    Uuid::from_bytes(uuid_bytes)
}

/// Convert a serde_json Value to a pretty JSON string
pub fn pretty_json(value: &Value) -> String {
    serde_json::to_string_pretty(value).unwrap_or_else(|_| value.to_string())
}

/// Merge two JSON values (for options merging)
pub fn merge_json(base: &mut Value, update: Value) {
    match (base, update) {
        (Value::Object(base_map), Value::Object(update_map)) => {
            for (k, v) in update_map {
                merge_json(base_map.entry(k).or_insert(Value::Null), v);
            }
        }
        (base, update) => {
            *base = update;
        }
    }
}

/// Parse time range strings into Superset format
pub fn parse_time_range(range: &str) -> String {
    match range.to_lowercase().as_str() {
        "today" => "today".to_string(),
        "yesterday" => "yesterday".to_string(),
        "this week" | "this_week" => "this week".to_string(),
        "last week" | "last_week" => "last week".to_string(),
        "this month" | "this_month" => "this month".to_string(),
        "last month" | "last_month" => "last month".to_string(),
        "this quarter" | "this_quarter" => "this quarter".to_string(),
        "last quarter" | "last_quarter" => "last quarter".to_string(),
        "this year" | "this_year" => "this year".to_string(),
        "last year" | "last_year" => "last year".to_string(),
        s if s.contains("last") && s.contains("day") => {
            // Parse "last X days" format
            let parts: Vec<&str> = s.split_whitespace().collect();
            if parts.len() >= 3 {
                if let Ok(num) = parts[1].parse::<i32>() {
                    return format!("last {} days", num);
                }
            }
            range.to_string()
        }
        _ => range.to_string(),
    }
}

/// Convert PostgreSQL column types to Superset-friendly types
pub fn normalize_column_type(pg_type: &str) -> String {
    match pg_type.to_uppercase().as_str() {
        "INT" | "INT2" | "INT4" | "INT8" | "SMALLINT" | "BIGINT" | "SERIAL" | "BIGSERIAL" => {
            "INTEGER".to_string()
        }
        "NUMERIC" | "DECIMAL" | "REAL" | "DOUBLE PRECISION" | "FLOAT" | "FLOAT4" | "FLOAT8" => {
            "DECIMAL".to_string()
        }
        "CHAR" | "CHARACTER" | "VARCHAR" | "CHARACTER VARYING" | "TEXT" => "VARCHAR".to_string(),
        "BOOL" | "BOOLEAN" => "BOOLEAN".to_string(),
        "DATE" => "DATE".to_string(),
        "TIME" | "TIMETZ" | "TIME WITH TIME ZONE" | "TIME WITHOUT TIME ZONE" => "TIME".to_string(),
        "TIMESTAMP" | "TIMESTAMPTZ" | "TIMESTAMP WITH TIME ZONE"
        | "TIMESTAMP WITHOUT TIME ZONE" => "TIMESTAMP".to_string(),
        "JSON" | "JSONB" => "JSON".to_string(),
        "UUID" => "UUID".to_string(),
        "BYTEA" => "BYTEA".to_string(),
        "ARRAY" => "ARRAY".to_string(),
        _ => pg_type.to_uppercase(),
    }
}

/// Generate a safe identifier from a name
pub fn safe_identifier(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_alphanumeric() || c == '_' { c } else { '_' })
        .collect::<String>()
        .to_lowercase()
}

/// Create default chart options based on chart type
pub fn default_chart_options(chart_type: &str) -> Value {
    match chart_type {
        "line" => json!({
            "show_legend": true,
            "show_markers": false,
            "line_interpolation": "linear",
            "x_axis_format": "smart_date",
            "y_axis_format": ",.0f",
            "rich_tooltip": true
        }),
        "bar" => json!({
            "show_legend": true,
            "show_bar_value": true,
            "bar_stacked": false,
            "y_axis_format": ",.0f"
        }),
        "pie" => json!({
            "donut": false,
            "show_labels": true,
            "label_type": "key_percent",
            "number_format": ",.0f"
        }),
        "table" => json!({
            "page_size": 25,
            "include_search": true,
            "show_cell_bars": true,
            "table_timestamp_format": "smart_date"
        }),
        "big_number_total" => json!({
            "compare_lag": 0,
            "compare_suffix": "",
            "show_trend_line": true,
            "start_y_axis_at_zero": true
        }),
        _ => json!({}),
    }
}

/// Validate and format SQL expressions
pub fn validate_sql_expression(expr: &str) -> Result<String, String> {
    let expr = expr.trim();

    // Basic validation
    if expr.is_empty() {
        return Err("Expression cannot be empty".to_string());
    }

    // Check for basic SQL injection patterns (very basic)
    let forbidden = [
        "--", "/*", "*/", ";", "drop", "delete", "truncate", "exec", "execute",
    ];
    let expr_lower = expr.to_lowercase();

    for pattern in &forbidden {
        if expr_lower.contains(pattern) {
            return Err(format!(
                "Expression contains forbidden pattern: {}",
                pattern
            ));
        }
    }

    Ok(expr.to_string())
}

/// Generate metric expression with proper aggregation
pub fn build_metric_expression(agg_type: &str, column: &str) -> String {
    match agg_type.to_uppercase().as_str() {
        "COUNT" => format!("COUNT(DISTINCT {})", column),
        "SUM" => format!("SUM({})", column),
        "AVG" => format!("AVG({})", column),
        "MIN" => format!("MIN({})", column),
        "MAX" => format!("MAX({})", column),
        "COUNT_ALL" => "COUNT(*)".to_string(),
        _ => format!("{}({})", agg_type.to_uppercase(), column),
    }
}

/// Format numbers for display
pub fn get_number_format(metric_type: &str, is_currency: bool) -> String {
    if is_currency {
        return "$,.2f".to_string();
    }

    match metric_type {
        "count" | "count_distinct" => ",.0f",
        "sum" if is_currency => "$,.2f",
        "sum" => ",.0f",
        "avg" | "mean" => ",.2f",
        "percentage" | "percent" => ",.1%",
        _ => ",.0f",
    }
    .to_string()
}

/// Extract dimension references from strings like "customer.name"
pub fn parse_dimension_reference(reference: &str) -> (Option<String>, String) {
    let parts: Vec<&str> = reference.split('.').collect();
    match parts.len() {
        1 => (None, parts[0].to_string()),
        2 => (Some(parts[0].to_string()), parts[1].to_string()),
        _ => (None, reference.to_string()),
    }
}

/// Validate SQL for virtual datasets
pub fn validate_virtual_dataset_sql(sql: &str) -> Result<String, String> {
    let sql = sql.trim();

    // Basic validation
    if sql.is_empty() {
        return Err("SQL query cannot be empty".to_string());
    }

    // Check it's a SELECT statement
    let sql_upper = sql.to_uppercase();
    let first_keyword = sql_upper.split_whitespace().next().unwrap_or("");
    if first_keyword != "SELECT" && first_keyword != "WITH" {
        return Err(
            "Virtual dataset SQL must be a SELECT statement or start with WITH (CTE)".to_string(),
        );
    }

    // Check for dangerous patterns
    let forbidden_keywords = [
        "INSERT", "UPDATE", "DELETE", "DROP", "CREATE", "ALTER", "TRUNCATE", "GRANT", "REVOKE",
        "EXECUTE", "EXEC",
    ];

    for keyword in &forbidden_keywords {
        // Simple word boundary check
        let pattern = format!(" {} ", keyword);
        let pattern_start = format!("{} ", keyword);
        if sql_upper.contains(&pattern) || sql_upper.starts_with(&pattern_start) {
            return Err(format!(
                "Virtual dataset SQL cannot contain {} statements",
                keyword
            ));
        }
    }

    // Validate parentheses balance
    let open_parens = sql.chars().filter(|&c| c == '(').count();
    let close_parens = sql.chars().filter(|&c| c == ')').count();
    if open_parens != close_parens {
        return Err("Unbalanced parentheses in SQL".to_string());
    }

    // Check for SQL injection patterns
    if sql.contains("--") || sql.contains("/*") || sql.contains("*/") {
        return Err("SQL comments are not allowed in virtual dataset queries".to_string());
    }

    // Check for multiple statements
    let statement_count = sql.matches(';').count();
    if statement_count > 0 && !sql.trim().ends_with(';') {
        return Err("Multiple SQL statements are not allowed".to_string());
    }

    Ok(sql.to_string())
}

/// Helper function to get time grain format for Superset
pub fn format_time_grain(grain: &str) -> String {
    match grain.to_lowercase().as_str() {
        "second" => "PT1S",
        "minute" => "PT1M",
        "hour" => "PT1H",
        "day" => "P1D",
        "week" => "P1W",
        "month" => "P1M",
        "quarter" => "P3M",
        "year" => "P1Y",
        _ => grain,
    }
    .to_string()
}

/// Helper function to map chart types
pub fn map_chart_type(chart_type: &str) -> String {
    match chart_type {
        "line" => "line",
        "bar" => "bar",
        "pie" => "pie",
        "table" => "table",
        "big_number" | "kpi" => "big_number_total",
        "area" => "area",
        "scatter" => "scatter",
        "bubble" => "bubble",
        "heatmap" => "heatmap",
        "box_plot" => "box_plot",
        "sunburst" => "sunburst",
        "sankey" => "sankey",
        "word_cloud" => "word_cloud",
        "treemap" => "treemap",
        "map" => "deck_geojson",
        _ => chart_type,
    }
    .to_string()
}

/// SQL Builder for common patterns
pub struct SqlBuilder;

impl SqlBuilder {
    /// Build a basic aggregation query
    pub fn aggregation_query(
        table: &str,
        group_by: Vec<&str>,
        aggregations: Vec<(&str, &str, &str)>, // (agg_func, column, alias)
        time_column: Option<&str>,
        time_grain: Option<&str>,
        filters: Vec<&str>,
    ) -> String {
        let mut sql = String::from("SELECT\n");

        // Add time column with truncation if specified
        if let Some(time_col) = time_column {
            let grain = time_grain.unwrap_or("day");
            sql.push_str(&format!(
                "  DATE_TRUNC('{}', {}) as date,\n",
                grain, time_col
            ));
        }

        // Add group by columns
        for col in &group_by {
            sql.push_str(&format!("  {},\n", col));
        }

        // Add aggregations
        for (i, (func, col, alias)) in aggregations.iter().enumerate() {
            if i == aggregations.len() - 1 && group_by.is_empty() && time_column.is_none() {
                sql.push_str(&format!("  {}({}) as {}\n", func, col, alias));
            } else {
                sql.push_str(&format!("  {}({}) as {},\n", func, col, alias));
            }
        }

        // Remove trailing comma and newline if needed
        if sql.ends_with(",\n") {
            sql.truncate(sql.len() - 2);
            sql.push('\n');
        }

        sql.push_str(&format!("FROM {}\n", table));

        // Add filters
        if !filters.is_empty() {
            sql.push_str("WHERE\n");
            for (i, filter) in filters.iter().enumerate() {
                if i == 0 {
                    sql.push_str(&format!("  {}\n", filter));
                } else {
                    sql.push_str(&format!("  AND {}\n", filter));
                }
            }
        }

        // Add group by clause
        let mut group_by_items = Vec::new();
        if time_column.is_some() {
            group_by_items.push("1".to_string()); // date column
        }
        for (i, _) in group_by.iter().enumerate() {
            group_by_items.push((i + 2).to_string());
        }

        if !group_by_items.is_empty() {
            sql.push_str(&format!("GROUP BY {}\n", group_by_items.join(", ")));
        }

        // Add order by date if time column exists
        if time_column.is_some() {
            sql.push_str("ORDER BY 1 DESC");
        }

        sql
    }

    /// Build a join query between two tables
    pub fn join_query(
        base_table: &str,
        base_alias: &str,
        join_table: &str,
        join_alias: &str,
        join_on: (&str, &str), // (base_column, join_column)
        select_columns: Vec<(&str, &str)>, // (table_alias, column)
        join_type: &str,       // "INNER", "LEFT", "RIGHT", "FULL"
    ) -> String {
        let mut sql = String::from("SELECT\n");

        // Add select columns
        for (i, (alias, col)) in select_columns.iter().enumerate() {
            if i == select_columns.len() - 1 {
                sql.push_str(&format!("  {}.{}\n", alias, col));
            } else {
                sql.push_str(&format!("  {}.{},\n", alias, col));
            }
        }

        // Add FROM clause
        sql.push_str(&format!("FROM {} {}\n", base_table, base_alias));

        // Add JOIN clause
        sql.push_str(&format!(
            "{} JOIN {} {}\n",
            join_type, join_table, join_alias
        ));
        sql.push_str(&format!(
            "  ON {}.{} = {}.{}",
            base_alias, join_on.0, join_alias, join_on.1
        ));

        sql
    }
}

/// Infer metric type from expression
pub fn infer_metric_type(expression: &str) -> &'static str {
    let expr_upper = expression.to_uppercase();
    if expr_upper.starts_with("COUNT") {
        "count"
    } else if expr_upper.starts_with("SUM") {
        "sum"
    } else if expr_upper.starts_with("AVG") {
        "avg"
    } else if expr_upper.starts_with("MIN") {
        "min"
    } else if expr_upper.starts_with("MAX") {
        "max"
    } else {
        ""
    }
}

/// Sanitize filename for filesystem
pub fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            _ => c,
        })
        .collect::<String>()
        .replace(' ', "_")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uuid_from_string() {
        let uuid1 = uuid_from_string("test");
        let uuid2 = uuid_from_string("test");
        let uuid3 = uuid_from_string("different");

        assert_eq!(uuid1, uuid2);
        assert_ne!(uuid1, uuid3);
    }

    #[test]
    fn test_parse_time_range() {
        assert_eq!(parse_time_range("last 7 days"), "last 7 days");
        assert_eq!(parse_time_range("this week"), "this week");
        assert_eq!(parse_time_range("last_month"), "last month");
    }

    #[test]
    fn test_normalize_column_type() {
        assert_eq!(normalize_column_type("int4"), "INTEGER");
        assert_eq!(normalize_column_type("varchar"), "VARCHAR");
        assert_eq!(normalize_column_type("timestamp"), "TIMESTAMP");
        assert_eq!(normalize_column_type("custom_type"), "CUSTOM_TYPE");
    }

    #[test]
    fn test_safe_identifier() {
        assert_eq!(safe_identifier("User Name"), "user_name");
        assert_eq!(safe_identifier("email@address"), "email_address");
        assert_eq!(safe_identifier("123_test"), "123_test");
    }

    #[test]
    fn test_validate_sql_expression() {
        assert!(validate_sql_expression("SUM(amount)").is_ok());
        assert!(validate_sql_expression("COUNT(*)").is_ok());
        assert!(validate_sql_expression("DROP TABLE users").is_err());
    }

    #[test]
    fn test_validate_virtual_dataset_sql() {
        // Valid SQL
        assert!(validate_virtual_dataset_sql("SELECT * FROM users").is_ok());
        assert!(validate_virtual_dataset_sql(
            "SELECT id, name FROM users WHERE active = true"
        )
        .is_ok());

        // Invalid SQL - empty
        assert!(validate_virtual_dataset_sql("").is_err());

        // Invalid SQL - not SELECT
        assert!(validate_virtual_dataset_sql("INSERT INTO users VALUES (1)").is_err());
    }

    #[test]
    fn test_format_time_grain() {
        assert_eq!(format_time_grain("month"), "P1M");
        assert_eq!(format_time_grain("day"), "P1D");
        assert_eq!(format_time_grain("custom"), "custom");
    }

    #[test]
    fn test_map_chart_type() {
        assert_eq!(map_chart_type("line"), "line");
        assert_eq!(map_chart_type("big_number"), "big_number_total");
        assert_eq!(map_chart_type("kpi"), "big_number_total");
    }

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("test/file:name"), "test_file_name");
        assert_eq!(sanitize_filename("normal_name"), "normal_name");
    }

    #[test]
    fn test_infer_metric_type() {
        assert_eq!(infer_metric_type("COUNT(*)"), "count");
        assert_eq!(infer_metric_type("SUM(amount)"), "sum");
        assert_eq!(infer_metric_type("AVG(price)"), "avg");
        assert_eq!(infer_metric_type("custom_expression"), "");
    }
}
