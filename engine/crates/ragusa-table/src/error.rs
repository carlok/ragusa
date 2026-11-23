use std::fmt;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum LoadError {
    Io { path: PathBuf, source: io::Error },
    Format { path: PathBuf, message: String },
}

impl LoadError {
    pub(crate) fn io(path: &Path, source: io::Error) -> Self {
        LoadError::Io {
            path: path.to_path_buf(),
            source,
        }
    }

    pub(crate) fn format(path: &Path, message: impl Into<String>) -> Self {
        LoadError::Format {
            path: path.to_path_buf(),
            message: message.into(),
        }
    }
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoadError::Io { path, source } => write!(f, "{}: {source}", path.display()),
            LoadError::Format { path, message } => write!(f, "{}: {message}", path.display()),
        }
    }
}

impl std::error::Error for LoadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            LoadError::Io { source, .. } => Some(source),
            LoadError::Format { .. } => None,
        }
    }
}
