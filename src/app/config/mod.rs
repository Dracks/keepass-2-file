pub mod yaml;

use std::{fmt::Display, path::PathBuf};

use yaml::YamlConfig;

#[derive(Debug)]
pub struct ConfigError {
    msg: String,
}
impl std::error::Error for ConfigError {}
impl Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl ConfigError {
    fn new(msg: impl Into<String>) -> Self {
        ConfigError { msg: msg.into() }
    }
}

pub struct ConfigHandler {
    file: String,
    project: String,
    pub global: YamlConfig,
    pub local: Option<YamlConfig>,
}

const PROJECT_FILE: &str = ".keepass-2-file";

impl ConfigHandler {
    fn get_project_file(project: &String) -> Option<PathBuf> {
        let mut project_path = std::path::Path::new(project).to_path_buf();
        if project_path.is_dir() {
            project_path.push(PROJECT_FILE);
            return Some(project_path);
        }
        None
    }

    pub fn new(file: &str, project: String) -> Result<ConfigHandler, Box<dyn std::error::Error>> {
        if !std::path::Path::new(file).exists() {
            std::fs::write(file, "")?;
        }

        let contents = std::fs::read_to_string(file)?;

        let config = YamlConfig::new(&contents)?;

        let local_config_file = Self::get_project_file(&project);

        let local: Option<YamlConfig> = match local_config_file {
            Some(config_path) => {
                if config_path.exists() && config_path.is_file() {
                    let config = config_path.into_os_string().into_string().map_err(|err| {
                        ConfigError::new(format!("Path containing invalid UTF-8 code: {:?}", err))
                    })?;
                    let contents = std::fs::read_to_string(config)?;
                    Some(YamlConfig::new(&contents)?)
                } else {
                    None
                }
            }
            None => None,
        };

        Ok(ConfigHandler {
            file: file.to_string(),
            global: config,
            local,
            project,
        })
    }

    pub fn get_file(&self) -> &str {
        &self.file
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let global_content = self.global.yaml()?;
        std::fs::write(&self.file, global_content)?;
        if let Some(local) = &self.local {
            let local_content = local.yaml()?;
            let Some(local_file) = Self::get_project_file(&self.project) else {
                return Err(ConfigError::new( "Cannot get the local project file, any changes in the project config won't be saved").into());
            };
            std::fs::write(local_file, local_content)?;
        }
        Ok(())
    }
}

pub enum SourceConfig {
    Project,
    Global,
}

impl ConfigHandler {
    pub fn keepass(&self) -> Option<String> {
        let local_config = self
            .local
            .as_ref()
            .map(|config| config.keepass.clone())
            .flatten();

        match local_config {
            Some(keepass) => Some(keepass),
            None => self.global.keepass.clone(),
        }
    }

    pub fn add_template(
        &mut self,
        destination: SourceConfig,
        name: Option<String>,
        template: String,
        output: String,
    ) -> Result<(), ConfigError> {
        match destination {
            SourceConfig::Global => Ok(self.global.add_template(name, template, output)),
            SourceConfig::Project => {
                let project = self.project.clone();
                let mut local = self.local.clone().unwrap_or(YamlConfig::default());
                let Some(relative_template) = pathdiff::diff_paths(&template, &project) else {
                    return Err(ConfigError::new("Cannot convert template ({}) to relative").into());
                };
                let Some(relative_output) = pathdiff::diff_paths(output, &project) else {
                    return Err(ConfigError::new("Cannot convert output ({}) to relative").into());
                };
                local.add_template(
                    name,
                    relative_template
                        .into_os_string()
                        .into_string()
                        .map_err(|err| {
                            ConfigError::new(&format!(
                                "Template path contains invalid UTF-8: {:?}",
                                err
                            ))
                        })?,
                    relative_output
                        .into_os_string()
                        .into_string()
                        .map_err(|err| {
                            ConfigError::new(&format!(
                                "Output path contains invalid UTF-8: {:?}",
                                err
                            ))
                        })?,
                );
                self.local = Some(local);
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::app::{config::PROJECT_FILE, test_helpers::tests::TestConfig};

    #[test]
    fn test_loading_empty_file() {
        let test_config = TestConfig::create_empty_file();
        std::fs::write(test_config.get_file_path(), "{}").expect("Can write test file");

        let config = test_config.get();

        assert_eq!(config.global.get_templates().len(), 0);
    }

    #[test]
    fn test_keepass_from_project() {
        let test_config = TestConfig::create();
        std::fs::write(
            format!("{}/{}", test_config.get_project_path(), PROJECT_FILE),
            "version: 2\nkeepass: /some/stupid/url.keepass",
        )
        .expect("We have a default for the project");

        let config = test_config.get();

        assert_eq!(config.keepass().unwrap(), "/some/stupid/url.keepass")
    }

    #[test]
    fn test_keepass_from_global() {
        let test_config = TestConfig::create();
        std::fs::write(
            format!("{}/{}", test_config.get_project_path(), PROJECT_FILE),
            "version: 2\n",
        )
        .expect("We have a default for the project");

        let config = test_config.get();

        assert!(
            config
                .keepass()
                .unwrap()
                .ends_with("test_resources/test_db.kdbx")
        )
    }

    #[test]
    fn test_keepass_from_global_without_project() {
        let test_config = TestConfig::create();

        let config = test_config.get();

        assert!(
            config
                .keepass()
                .unwrap()
                .ends_with("test_resources/test_db.kdbx")
        )
    }
}
