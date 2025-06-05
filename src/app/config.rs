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

    #[serde(default)]
    templates: Vec<YamlConfigTemplate>,
    #[serde(default)]
    variables: HashMap<String, String>,
}

impl YamlConfig {
    pub fn get_templates(&self) -> Vec<YamlConfigTemplate> {
        self.templates.to_vec()
    }

    pub fn add_template(
        &mut self,
        name: Option<String>,
        template_path: String,
        output_path: String,
    ) {
        let mut templates = self.templates.clone();
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
        self.templates = templates
    }

    pub fn delete_templates(&mut self, name: String) {
        let templates = self.templates.clone().into_iter();
        self.templates = templates
            .filter(|template| {
                if let Some(template_name) = &template.name {
                    return &name != template_name;
                }
                true
            })
            .collect();
    }

    pub fn delete_template(&mut self, template_path: String, output_path: String) {
        let templates = self.templates.clone().into_iter();
        self.templates = templates
            .filter(|template| {
                !(template.template_path == template_path && template.output_path == output_path)
            })
            .collect();
    }

    pub fn get_vars(&self) -> HashMap<String, String> {
        self.variables.clone()
    }

    pub fn add_var(&mut self, var_name: String, value: String) {
        let mut tmp_variables = self.variables.clone();
        tmp_variables.insert(var_name, value);
        self.variables = tmp_variables
    }

    pub fn del_var(&mut self, var_name: String) {
        let mut tmp_variables = self.variables.clone();
        tmp_variables.remove(&var_name);
        self.variables = tmp_variables
    }
}

pub struct ConfigHandler<'f> {
    file: &'f str,
    pub config: YamlConfig,
}

impl ConfigHandler<'_> {
    pub fn new(file: &str) -> Result<ConfigHandler, Box<dyn std::error::Error>> {
        if !std::path::Path::new(file).exists() {
            std::fs::write(file, "")?;
        }

        let contents = std::fs::read_to_string(file)?;
        let config = if contents.is_empty() {
            YamlConfig {
                keepass: None,
                templates: Vec::new(),
                variables: HashMap::new(),
            }
        } else {
            serde_yaml::from_str(&contents)?
        };

        Ok(ConfigHandler { file, config })
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

#[cfg(test)]
mod test {
    use crate::app::test_helpers::tests::TestConfig;

    #[test]
    fn test_loading_empty_file() {
        let test_config = TestConfig::create_empty_file();
        std::fs::write(test_config.get_file_path(), "{}").expect("Can write test file");

        let config = test_config.get();

        assert_eq!(config.config.get_templates().len(), 0);
    }
}
