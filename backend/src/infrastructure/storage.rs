use std::path::{Path, PathBuf};

use tokio::fs;

use crate::shared::errors::AppError;

/// Creates the storage root if missing. Called once during bootstrap.
pub fn init(root: PathBuf) -> Result<Storage, String> {
    std::fs::create_dir_all(root.join("attachments"))
        .map_err(|e| format!("cannot create storage root {root:?}: {e}"))?;
    Ok(Storage::new(root))
}

/// On-disk storage for attachment bytes. Metadata stays in PostgreSQL.
#[derive(Debug, Clone)]
pub struct Storage {
    root: PathBuf,
}

impl Storage {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    /// Stores bytes under `attachments/<user>/<uuid><ext>` and returns the relative path.
    pub async fn save(
        &self,
        user_id: uuid::Uuid,
        file_name: &str,
        bytes: &[u8],
    ) -> Result<String, AppError> {
        let dir = self.root.join("attachments").join(user_id.to_string());
        fs::create_dir_all(&dir).await?;

        let ext = Path::new(file_name)
            .extension()
            .map(|e| e.to_string_lossy().to_lowercase())
            .filter(|e| e.len() <= 8 && e.chars().all(|c| c.is_ascii_alphanumeric()))
            .map(|e| format!(".{e}"))
            .unwrap_or_default();
        let stored = format!("attachments/{}/{}{}", user_id, uuid::Uuid::new_v4(), ext);

        fs::write(self.root.join(&stored), bytes).await?;
        Ok(stored)
    }

    pub async fn read(&self, relative_path: &str) -> Result<Vec<u8>, AppError> {
        Ok(fs::read(self.absolute(relative_path)).await?)
    }

    pub async fn delete(&self, relative_path: &str) -> Result<(), AppError> {
        match fs::remove_file(self.absolute(relative_path)).await {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(e.into()),
        }
    }

    /// Rejects any path that escapes the storage root.
    fn absolute(&self, relative_path: &str) -> PathBuf {
        let candidate = self.root.join(relative_path);
        if candidate.starts_with(&self.root) {
            candidate
        } else {
            self.root.join("invalid")
        }
    }
}
