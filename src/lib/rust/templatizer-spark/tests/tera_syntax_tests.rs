#[cfg(test)]
mod tera_syntax_tests {
    use tera::{Tera, Context};
    use serde_json::json;

    #[test]
    fn test_conditional_vs_default_filter() {
        let mut tera = Tera::default();
        
        // Test that conditional syntax works (our fix)
        tera.add_raw_template("conditional.tera", r#"
Level: {% if log_level %}{{ log_level }}{% else %}INFO{% endif %}
Name: {% if name %}{{ name }}{% else %}DefaultName{% endif %}
"#).unwrap();
        
        // Test with values present
        let context = Context::from_serialize(json!({
            "log_level": "DEBUG",
            "name": "TestApp"
        })).unwrap();
        
        let result = tera.render("conditional.tera", &context).unwrap();
        assert!(result.contains("Level: DEBUG"));
        assert!(result.contains("Name: TestApp"));
        
        // Test with values missing
        let empty_context = Context::new();
        let result = tera.render("conditional.tera", &empty_context).unwrap();
        assert!(result.contains("Level: INFO"));
        assert!(result.contains("Name: DefaultName"));
        
        println!("✅ Conditional syntax test passed!");
    }
    
    #[test]
    fn test_array_access_in_loops() {
        let mut tera = Tera::default();
        
        tera.add_raw_template("array_loop.tera", r#"
{% for item in fields %}
Field: {{ item[0] }}, Type: {{ item[1] }}
{% endfor %}
"#).unwrap();
        
        let context = Context::from_serialize(json!({
            "fields": [
                ["name", "String"],
                ["age", "Integer"],
                ["active", "Boolean"]
            ]
        })).unwrap();
        
        let result = tera.render("array_loop.tera", &context).unwrap();
        assert!(result.contains("Field: name, Type: String"));
        assert!(result.contains("Field: age, Type: Integer"));
        assert!(result.contains("Field: active, Type: Boolean"));
        
        println!("✅ Array access in loops test passed!");
    }
    
    #[test]
    fn test_join_filter_syntax() {
        let mut tera = Tera::default();
        
        tera.add_raw_template("join.tera", r#"
Args: {{ args | join(", ") }}
"#).unwrap();
        
        let context = Context::from_serialize(json!({
            "args": ["arg1", "arg2", "arg3"]
        })).unwrap();
        
        let result = tera.render("join.tera", &context).unwrap();
        assert!(result.contains("Args: arg1, arg2, arg3"));
        
        println!("✅ Join filter test passed!");
    }
    
    #[test]
    fn test_complex_conditionals() {
        let mut tera = Tera::default();
        
        // Test nested conditionals (our fix for complex boolean expressions)
        tera.add_raw_template("nested.tera", r#"
{% if config_enabled %}
  {% if has_fields %}
    Config is enabled and has fields
  {% else %}
    Config is enabled but no fields
  {% endif %}
{% else %}
  Config is disabled
{% endif %}
"#).unwrap();
        
        // Test all combinations
        let cases = vec![
            (true, true, "Config is enabled and has fields"),
            (true, false, "Config is enabled but no fields"),
            (false, true, "Config is disabled"),
            (false, false, "Config is disabled"),
        ];
        
        for (config, fields, expected) in cases {
            let context = Context::from_serialize(json!({
                "config_enabled": config,
                "has_fields": fields
            })).unwrap();
            
            let result = tera.render("nested.tera", &context).unwrap();
            assert!(result.contains(expected));
        }
        
        println!("✅ Complex conditionals test passed!");
    }
}
