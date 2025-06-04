#[cfg(test)]
pub mod tests {
    use super::super::config::GlobalConfig;
    use super::super::tools::normalize_separators;
    use super::super::IOLogs;
    use std::cell::RefCell;
    use std::path::Path;

    pub struct TestConfig {
        config_file: String,
        auto_clean: bool,
    }

    impl TestConfig {
        pub fn get_file_path(&self) -> String {
            self.config_file.clone()
        }

        pub fn get(&self) -> GlobalConfig {
            GlobalConfig::new(&self.config_file)
                .expect("Failed to load temp config created by TestConfig")
        }

        pub fn create_super_config() -> TestConfig {
            let uuid = uuid::Uuid::new_v4();
            let config_file = format!("test_resources/tmp/config_{}.yml", uuid);
            let current_path = std::env::current_dir().unwrap();
            let current_path_string = current_path.to_str().unwrap();
            std::fs::create_dir_all("test_resources/tmp")
                .expect("Unable to create temporary test_resources/tmp directory");

            if let Ok(config) = GlobalConfig::new(&config_file) {
                let mut config: GlobalConfig<'_> = config;
                config.config.add_template(
                    Some("something".into()),
                    current_path_string.to_owned()
                        + &normalize_separators("/test_resources/test_db.kdbx"),
                    "something".into(),
                );

                let template = Path::new("test_resources/.env.example")
                    .canonicalize()
                    .expect("Testing!");
                let template_string: String = String::from(template.to_str().unwrap());
                let output = Path::new("test_resources/tmp/.env")
                    .canonicalize()
                    .expect("Testing!");

                config.config.add_template(
                    Some("valid".into()),
                    template_string,
                    String::from(output.to_str().unwrap()),
                );

                config.config.add_template(
                    Some("other".into()),
                    current_path_string.to_owned()
                        + &normalize_separators("/test_resources/.env.example"),
                    current_path_string.to_owned()
                        + &normalize_separators("/test_resources/tmp/.env2"),
                );
                let _ = config.save();
            }

            TestConfig {
                config_file,
                auto_clean: false,
            }
        }

        fn create_config(content: Option<String>) -> TestConfig {
            let uuid = uuid::Uuid::new_v4();
            let config_file = format!("test_resources/tmp/config_{}.yml", uuid);
            std::fs::create_dir_all("test_resources/tmp")
                .expect("Unable to create temporary test_resources/tmp directory");
            if let Some(content) = content {
                std::fs::write(&config_file, content)
                    .expect("Unable to write temporary configuration file");
            }
            TestConfig {
                config_file,
                auto_clean: true,
            }
        }

        pub fn create() -> TestConfig {
            let current_path = std::env::current_dir().unwrap();
            let current_path_string = current_path.to_str().unwrap();
            let test_path = current_path_string.to_owned()
                + &normalize_separators("/test_resources/.env.example");

            let test_config = format!(
                "keepass: {current_path_string}/test_resources/test_db.kdbx
templates:
- template_path: {current_path_string}/some-missing-file
  output_path: something
- template_path: {test_path}
  output_path: {current_path_string}/test_resources/tmp/.env
  name: valid
- template_path: {current_path_string}/test_resources/.env.example
  output_path: {current_path_string}/test_resources/tmp/.env2
  name: other
        "
            );
            TestConfig::create_config(Some(test_config))
        }

        #[allow(dead_code)]
        pub fn create_with_vars() -> TestConfig {
            let current_path = std::env::current_dir().unwrap();
            let current_path_display = current_path.display();

            let test_config = format!(
                "keepass: {current_path_display}/test_resources/test_db.kdbx
templates:
- template_path: {current_path_display}/test_resources/.env.example
  output_path: {current_path_display}/test_resources/tmp/.env
  name: valid
- template_path: {current_path_display}/test_resources/with_variables
  output_path: {current_path_display}/test_resources/tmp/with_variables
  name: valid
variables:
    something: is a variable
    email: j@k.com
        "
            );
            TestConfig::create_config(Some(test_config))
        }

        #[allow(dead_code)]
        pub fn create_with_errors() -> TestConfig {
            let current_path = std::env::current_dir().unwrap();
            let current_path_display = current_path.display();

            let test_config = format!(
                "keepass: {current_path_display}/test_resources/test_db.kdbx
templates:
- template_path: {current_path_display}/test_resources/0-with-errors
  output_path: {current_path_display}/test_resources/tmp/0.env
- template_path: {current_path_display}/test_resources/1-with-other-errors
  output_path: {current_path_display}/test_resources/tmp/1.env
- template_path: {current_path_display}/test_resources/.env.example
  output_path: {current_path_display}/test_resources/tmp/ok.env
variables:
    something: is a variable
    email: j@k.com
        "
            );
            TestConfig::create_config(Some(test_config))
        }

        pub fn create_empty_file() -> TestConfig {
            TestConfig::create_config(None)
        }

        #[allow(dead_code)]
        pub fn disable_auto_clean(&mut self) -> &TestConfig {
            self.auto_clean = false;
            self
        }
    }

    impl Drop for TestConfig {
        fn drop(&mut self) {
            if self.auto_clean {
                // Cleanup will happen even if test fails
                std::fs::remove_file(&self.config_file).unwrap_or_default();
            }
        }
    }

    pub struct IODebug {
        stdouts: RefCell<Vec<String>>,
        stdins: RefCell<Vec<String>>,
        stderrs: RefCell<Vec<String>>,
    }

    impl IODebug {
        pub fn new() -> IODebug {
            IODebug {
                stdins: RefCell::new(Vec::new()),
                stderrs: RefCell::new(Vec::new()),
                stdouts: RefCell::new(Vec::new()),
            }
        }

        pub fn add_stdin(&mut self, input: String) -> &IODebug {
            self.stdins.borrow_mut().push(input);
            self
        }

        pub fn get_logs(&self) -> Vec<String> {
            self.stdouts.borrow().clone()
        }

        pub fn get_errors(&self) -> Vec<String> {
            self.stderrs.borrow().clone()
        }
    }

    impl IOLogs for IODebug {
        fn log(&self, str: String) {
            self.stdouts.borrow_mut().push(str);
        }

        fn read(&self, str: String, secure: bool) -> std::io::Result<String> {
            let mut stdins = self.stdins.borrow_mut();
            if !stdins.is_empty() {
                let value = stdins.remove(0);
                Ok(value)
            } else {
                panic!("Consuming stdin when is empty. prompt: {str} secure: {secure}");
            }
        }

        fn error(&self, str: String) {
            self.stderrs.borrow_mut().push(str);
        }
    }
}
