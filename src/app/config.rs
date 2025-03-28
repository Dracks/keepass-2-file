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
        templates.push(YamlConfigTemplate {
            name,
            template_path,
            output_path,
        });
        templates.sort_by(|a, b| a.template_path.cmp(&b.template_path));
        self.templates = Some(templates)
    }

    pub fn delete_templates(&mut self, name: String) {
        let templates = self.templates.clone();
        let templates = templates.unwrap_or_default().into_iter();
        self.templates = Some(
            templates
                .filter(|template| {
                    if let Some(template_name) = template.name.clone() {
                        return name != template_name;
                    }
                    false
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
