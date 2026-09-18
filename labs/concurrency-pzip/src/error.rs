use std::{collections::HashMap, error::Error, fmt::Display, sync::OnceLock};

pub type Result<T> = core::result::Result<T, AppError>;

pub struct AppError(Box<dyn Display>);

impl AppError {
    pub fn new(msg: impl Display + 'static) -> Self {
        AppError(Box::new(msg))
    }
}

impl Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::fmt::Debug for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}

impl<T> From<T> for AppError
where
    T: Error + 'static,
{
    fn from(value: T) -> Self {
        AppError(Box::new(value))
    }
}
