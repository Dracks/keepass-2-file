use std::path::MAIN_SEPARATOR;

pub fn convert_vecs<T, U>(v: Vec<T>) -> Vec<U>
where
    T: Into<U>,
{
    v.into_iter().map(Into::into).collect()
}

#[allow(dead_code)]
pub fn normalize_separators(path: &str) -> String {
    path.replace(['/', '\\'], &MAIN_SEPARATOR.to_string())
}
