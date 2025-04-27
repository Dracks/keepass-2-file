#[cfg(test)]
pub mod tests {
    use super::super::config::GlobalConfig;

    pub struct TestConfig {
        config_file: String,
        auto_clean: bool,
    }

    impl TestConfig {
        fn write_config(&self, content: String) {
            std::fs::create_dir_all("test_resources/tmp")
                .expect("Unable to create temporary test_resources/tmp directory");
            std::fs::write(&self.config_file, content)
                .expect("Unable to write temporary configuration file");
        }

        pub fn get_file_path(&self) -> String {
            self.config_file.clone()
        }

        pub fn get(&self) -> GlobalConfig {
            GlobalConfig::new(&self.config_file)
+        .expect("Failed to load temp config created by TestConfig")
        }

        #[allow(dead_code)]
        pub fn create() -> TestConfig {
            let uuid = uuid::Uuid::new_v4();
            let config_file = format!("test_resources/tmp/config_{}.yml", uuid);
            let current_path = std::env::current_dir().unwrap();
            let current_path_display = current_path.display();

            let test_config = format!(
                "keepass: {current_path_display}/test_resources/test_db.kdbx
templates:
- template_path: {current_path_display}/some-missing-file
  output_path: something
- template_path: {current_path_display}/test_resources/.env.example
  output_path: {current_path_display}/test_resources/tmp/.env
  name: valid
- template_path: {current_path_display}/test_resources/.env.example
  output_path: {current_path_display}/test_resources/tmp/.env2
  name: other
        "
            );
            let instance = TestConfig {
                config_file,
                auto_clean: true,
            };
            instance.write_config(test_config);
            instance
        }

        #[allow(dead_code)]
        pub fn create_with_vars() -> TestConfig {
            let uuid = uuid::Uuid::new_v4();
            let config_file = format!("test_resources/tmp/config_{}.yml", uuid);
            let current_path = std::env::current_dir().unwrap();
            let current_path_display = current_path.display();

            let test_config = format!(
                "keepass: {current_path_display}/test_resources/test_db.kdbx
templates:
- template_path: {current_path_display}/test_resources/.env.example
  output_path: {current_path_display}/test_resources/tmp/.env
  name: valid
variables:
    something: is a variable
    email: j@k.com
        "
            );
            let instance = TestConfig {
                config_file,
                auto_clean: true,
            };
            instance.write_config(test_config);
            instance
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
}
