use std::path::PathBuf;
use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use snowcloud_flake::serde_ext::string_id;

use crate::ids;

#[derive(Debug, Serialize, Deserialize)]
pub struct StorageLocal {
    pub path: PathBuf
}

#[derive(Debug, Serialize, Deserialize)]
pub enum StorageType {
    Local(StorageLocal)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StorageListItem {
    #[serde(with = "string_id")]
    pub id: ids::StorageId,
    pub name: String,
    #[serde(with = "string_id")]
    pub user_id: ids::UserId,
    #[serde(rename(serialize = "type", deserialize = "type"))]
    pub type_: StorageType,
    pub tags: HashMap<String, Option<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StorageItem {
    #[serde(with = "string_id")]
    pub id: ids::StorageId,
    pub name: String,
    #[serde(with = "string_id")]
    pub user_id: ids::UserId,
    #[serde(rename(serialize = "type", deserialize = "type"))]
    pub type_: StorageType,
    pub tags: HashMap<String, Option<String>>,
    pub created: DateTime<Utc>,
    pub updated: Option<DateTime<Utc>>,
    pub deleted: Option<DateTime<Utc>>,
}
