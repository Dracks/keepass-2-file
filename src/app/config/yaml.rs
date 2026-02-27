
use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_yaml::{self};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct YamlConfigTemplate {
    pub name: Option<String>,
    pub template_path: String,
    pub output_path: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct YamlConfig {
    pub keepass: Option<String>,

    #[serde(default)]
    templates: Vec<YamlConfigTemplate>,
    #[serde(default)]
    variables: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct YamlConfigV1 {
    pub keepass: Option<String>,

    #[serde(default)]
    templates: Option<Vec<YamlConfigTemplate>>,
    #[serde(default)]
    variables: Option<HashMap<String, String>>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "version")]
enum YamlConfigVersioned {
    #[serde(rename = "2")]
    Latest(YamlConfig),
    #[serde(untagged)]
    Legacy(YamlConfigV1), // For files without version field
}

impl YamlConfig {

    pub fn new(contents: &str) -> Result<Self, serde_yaml::Error> {

        if contents.is_empty() {
            Ok(YamlConfig::default())
        } else {
            let versioned: YamlConfigVersioned = serde_yaml::from_str(&contents)?;
            match versioned {
                YamlConfigVersioned::Latest(conf) => Ok(conf),
                YamlConfigVersioned::Legacy(legacy) => Ok(legacy.into()),
            }
        }
    }

    pub fn yaml(&self) -> Result<String, serde_yaml::Error> {
        let version = YamlConfigVersioned::Latest(self.clone());
        serde_yaml::to_string(&version)
    }

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

impl From<YamlConfigV1> for YamlConfig {
    fn from(value: YamlConfigV1) -> Self {
        Self {
            keepass: value.keepass,
            templates: value.templates.unwrap_or_default(),
            variables: value.variables.unwrap_or_default(),
        }
    }
}
