pub fn convert_vecs<T, U>(v: Vec<T>) -> Vec<U>
where
    T: Into<U>,
{
    v.into_iter().map(Into::into).collect()
}

#[cfg(test)]
pub mod tests {

    pub fn normalize_separators(path: &str) -> String {
        path.replace(['/', '\\'], std::path::MAIN_SEPARATOR_STR)
    }
}
