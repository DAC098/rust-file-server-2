use std::path::{PathBuf, Path};

use lib::models::storage::{StorageItem, StorageType, StorageLocal};
use serde::{Serialize, Deserialize};

use super::error::BuilderError;

#[derive(Debug, Serialize, Deserialize)]
pub struct Local {
    pub path: PathBuf,
}

impl Local {
    pub async fn build(path: PathBuf) -> Result<Self, BuilderError> {
        if path.try_exists()? {
            if !path.is_dir() {
                return Err(BuilderError::PathNotDirectory);
            }
        } else {
            tokio::fs::create_dir_all(&path).await?;
        }

        Ok(Local { path })
    }

    pub fn into_model(self) -> StorageLocal {
        StorageLocal {
            path: self.path
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Type {
    Local(Local)
}

impl Type {
    pub fn into_model(self) -> StorageType {
        match self {
            Type::Local(local) => StorageType::Local(local.into_model())
        }
    }
}

impl From<Local> for Type {
    fn from(local: Local) -> Self {
        Type::Local(local)
    }
}
