use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_yaml::{self};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct YamlConfigTemplate {
    pub name: Option<String>,
    pub template_path: String,
    pub output_path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct YamlConfig {
    pub keepass: Option<String>,
    templates: Option<Vec<YamlConfigTemplate>>,
    variables: Option<HashMap<String, String>>,
}

impl YamlConfig {
    pub fn get_templates(&self) -> Vec<YamlConfigTemplate> {
        if let Some(ref templates) = self.templates {
            templates.to_vec()
        } else {
            Vec::new()
        }
    }

    pub fn add_template(
        &mut self,
        name: Option<String>,
        template_path: String,
        output_path: String,
    ) {
        let templates = self.templates.clone();
        let mut templates = templates.unwrap_or_default();
        let existing_template = templates
            .iter_mut()
            .find(|t| t.template_path == template_path && t.output_path == output_path);

        if let Some(template) = existing_template {
            template.name = name;
        } else {
            templates.push(YamlConfigTemplate {
                name,
                template_path,
                output_path,
            });
        }
        templates.sort_by(|a, b| match a.template_path.cmp(&b.template_path) {
            std::cmp::Ordering::Equal => a.output_path.cmp(&b.output_path),
            ord => ord,
        });
        self.templates = Some(templates)
    }

    pub fn delete_templates(&mut self, name: String) {
        let templates = self.templates.clone();
        let templates = templates.unwrap_or_default().into_iter();
        self.templates = Some(
            templates
                .filter(|template| {
                    if let Some(template_name) = &template.name {
                        return &name != template_name;
                    }
                    true
                })
                .collect(),
        );
    }

    pub fn delete_template(&mut self, template_path: String, output_path: String) {
        let templates = self.templates.clone();
        let templates = templates.unwrap_or_default().into_iter();
        self.templates = Some(
            templates
                .filter(|template| {
                    !(template.template_path == template_path
                        && template.output_path == output_path)
                })
                .collect(),
        );
    }

    pub fn get_vars(&self) -> HashMap<String, String> {
        self.variables.clone().unwrap_or_default()
    }

    pub fn add_var(&mut self, var_name: String, value: String) {
        let temp_variables = self.variables.clone();
        let mut tmp_variables = temp_variables.unwrap_or_default();
        tmp_variables.insert(var_name, value);
        self.variables = Some(tmp_variables)
    }

    pub fn del_var(&mut self, var_name: String) {
        let temp_variables = self.variables.clone();
        let mut tmp_variables = temp_variables.unwrap_or_default();
        tmp_variables.remove(&var_name);
        self.variables = Some(tmp_variables)
    }
}

pub struct GlobalConfig<'f> {
    file: &'f str,
    pub config: YamlConfig,
}

impl GlobalConfig<'_> {
    pub fn new(file: &str) -> Result<GlobalConfig, Box<dyn std::error::Error>> {
        if !std::path::Path::new(file).exists() {
            std::fs::write(file, "")?;
        }

        let contents = std::fs::read_to_string(file)?;
        let config = if contents.is_empty() {
            YamlConfig {
                keepass: None,
                templates: None,
                variables: None,
            }
        } else {
            serde_yaml::from_str(&contents)?
        };

        Ok(GlobalConfig { file, config })
    }

    pub fn get_file(&self) -> &str {
        self.file
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let yaml = serde_yaml::to_string(&self.config)?;
        std::fs::write(self.file, yaml)?;
        Ok(())
    }
}
