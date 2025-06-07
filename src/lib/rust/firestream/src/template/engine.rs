use tera::{Tera, Context, Result as TeraResult};
use crate::template::embedded::TEMPLATES;
use crate::core::Result;

pub struct TemplateEngine {
    tera: Tera,
}

impl TemplateEngine {
    pub fn new() -> Result<Self> {
        let mut tera = Tera::default();
        
        // Add all embedded templates
        for (name, content) in TEMPLATES.iter() {
            tera.add_raw_template(name, content)?;
        }
        
        // The json_encode filter is built-in to Tera
        
        Ok(Self { tera })
    }
    
    pub fn render(&self, template_name: &str, context: &Context) -> TeraResult<String> {
        self.tera.render(template_name, context)
    }
}
