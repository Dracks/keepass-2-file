use std::{fs, path};

pub fn convert_vecs<T, U>(v: Vec<T>) -> Vec<U>
where
    T: Into<U>,
{
    v.into_iter().map(Into::into).collect()
}

pub enum NormalizedInput {
    Str(String),
    PathBuf(path::PathBuf),
}

impl From<&str> for NormalizedInput {
    fn from(s: &str) -> Self {
        NormalizedInput::Str(s.to_string())
    }
}

impl From<String> for NormalizedInput {
    fn from(s: String) -> Self {
        NormalizedInput::Str(s)
    }
}

impl From<path::PathBuf> for NormalizedInput {
    fn from(p: path::PathBuf) -> Self {
        NormalizedInput::PathBuf(p)
    }
}

pub fn normalize_path<P: Into<NormalizedInput>>(input: P) -> String {
    match input.into() {
        NormalizedInput::Str(s) => {
            let path = fs::canonicalize(path::Path::new(&s)).unwrap();
            let stringified = path.to_str().unwrap().to_string();
            return stringified;
        }
        NormalizedInput::PathBuf(pb) => {
            let path = fs::canonicalize(pb).unwrap();
            let stringified = path.to_str().unwrap().to_string();
            return stringified;
        }
    }
}
