pub struct TmpFile {
    file: String,
    auto_clean: bool,
}

impl TmpFile {
    pub fn new(file: String) -> Self {
        Self {
            file,
            auto_clean: true,
        }
    }

    pub fn new_uuid(path:impl Into<String>, ext:impl Into<String>) -> Self {
        let uuid = uuid::Uuid::new_v4();
        let path = path.into();
        let ext = ext.into();
        let config_file = format!("{path}/{uuid}.{ext}");
        let error = format!("Unable to create temporary {} directory", &path);
        std::fs::create_dir_all(path).expect(error.as_str());
        Self::new(config_file)
    }

    pub fn write(&self, contents: String) {
        std::fs::write(&self.file, contents).expect("Unable to write temporary configuration file");
    }

    pub fn read(&self) -> String {
        std::fs::read_to_string(&self.file).expect("Unable to read the file")
    }

    pub fn get(&self) -> String {
        self.file.clone()
    }

    pub fn disable_auto_clean(&mut self) {
        self.auto_clean = false;
    }
}

impl Drop for TmpFile {
    fn drop(&mut self) {
        if self.auto_clean {
            // Cleanup will happen even if test fails
            std::fs::remove_file(&self.file).unwrap_or_default();
        }
    }
}
