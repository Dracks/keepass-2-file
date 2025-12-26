mod tmp_file;
pub use tmp_file::TmpFile;

pub fn normalize_separators(path: &str) -> String {
    path.replace(['/', '\\'], std::path::MAIN_SEPARATOR_STR)
}
