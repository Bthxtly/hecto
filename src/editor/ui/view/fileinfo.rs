use std::{
    fmt::Display,
    path::{Path, PathBuf},
};

#[derive(Default)]
pub struct FileInfo {
    path: Option<PathBuf>,
}

impl FileInfo {
    pub fn from(filename: &str) -> Self {
        Self {
            path: Some(PathBuf::from(filename)),
        }
    }

    pub const fn has_path(&self) -> bool {
        self.path.is_some()
    }

    pub fn get_path(&self) -> Option<&Path> {
        self.path.as_deref()
    }
}

impl Display for FileInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = self
            .get_path()
            .and_then(|path| path.file_name())
            .and_then(|name| name.to_str())
            .unwrap_or("[No Name]");

        write!(f, "{name}")
    }
}
