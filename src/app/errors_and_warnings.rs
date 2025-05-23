use std::sync::{Arc, Mutex};

pub trait ErrorRecord: Send + Sync {
    fn register_error(&self, code: ErrorCode);
}

#[derive(Clone, Debug)]
pub enum ErrorCode {
    MissingEntry(Vec<String>),
    MissingField(Vec<String>, String),
    MissingPassword(Vec<String>),
    MissingUsername(Vec<String>),
    MissingUrl(Vec<String>),
}

#[derive(Clone)]
pub struct HelperErrors {
    errors: Arc<Mutex<Vec<ErrorCode>>>,
}

impl HelperErrors {
    pub fn new() -> Self {
        Self {
            errors: Arc::new(Mutex::new(vec![])),
        }
    }

    pub fn clean(&mut self) {
        let mut errors = self.errors.lock().unwrap();
        errors.clear();
    }

    pub fn get_errors(&self) -> Vec<ErrorCode> {
        let errors = self.errors.lock().unwrap();
        errors.clone()
    }
}

impl ErrorRecord for HelperErrors {
    fn register_error(&self, code: ErrorCode) {
        let mut errors = self.errors.lock().unwrap();
        errors.push(code);
    }
}
