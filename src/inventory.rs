use anyhow::{Context, Result};
use serde::Deserialize;
use std::{collections::HashSet, env};

#[derive(Deserialize, Debug)]
pub struct ArchiveItem {
    #[serde(rename = "ArchiveId")]
    archive_id: String,
}

#[derive(Deserialize, Debug)]
pub struct Inventory {
    #[serde(rename = "ArchiveList")]
    pub archive_list: Vec<ArchiveItem>,
}

impl Inventory {
    pub async fn new() -> Result<Self> {
        let file_path = env::var("INVENTORY_PATH").context("INVENTORY_PATH must be set")?;

        let file_contents = tokio::fs::read(file_path).await?;
        let inventory: Inventory = serde_json::from_slice(&file_contents)?;
        Ok(inventory)
    }
    pub fn filter_deleted(&mut self, deleted_ids: &HashSet<String>) {
        self.archive_list
            .retain(|item| !deleted_ids.contains(&item.archive_id));
    }
}

#[derive(Deserialize, Debug)]
pub struct DeletedInventory {
    pub ids: HashSet<String>,
}

impl DeletedInventory {
    pub async fn new() -> Result<Self> {
        let file_path = env::var("DELETED_PATH").context("DELETED_PATH must be set")?;
        let contents = match tokio::fs::read_to_string(&file_path).await {
            Ok(file_contents) => file_contents,

            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                tokio::fs::write(file_path, "").await?;
                String::new()
            }
            Err(e) => return Err(e.into()),
        };

        let ids = contents.lines().map(String::from).collect();

        Ok(Self { ids })
    }
}
