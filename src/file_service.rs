use anyhow::{Context, Result};
use serde::Deserialize;
use std::{collections::HashSet, env};
use tokio::io::AsyncWriteExt;

#[derive(Debug)]
pub struct FileService {
    pub delete_ids: Vec<String>,
    deleted_log_path: String,
}

impl FileService {
    pub async fn new() -> Result<Self> {
        let deleted_log_path = env::var("DELETED_PATH").context("DELETED_PATH must be set")?;
        let inventory_fut = Inventory::new();
        let deleted_inventory_fut = DeletedInventory::new(&deleted_log_path);

        let (inventory, deleted_inventory) =
            tokio::try_join!(inventory_fut, deleted_inventory_fut)?;

        let inventory_ids = inventory.get_id_set();

        let delete_ids: Vec<String> = inventory_ids
            .difference(&deleted_inventory.ids)
            .cloned()
            .collect();

        println!("Found {} total items in inventory.", inventory_ids.len());
        println!(
            "Found {} already deleted items.",
            deleted_inventory.ids.len()
        );
        println!("Found {} items that need to be deleted.", delete_ids.len());

        Ok(Self {
            delete_ids,
            deleted_log_path,
        })
    }

    pub async fn log_deleted_id(&self, id_to_log: &str) -> Result<()> {
        let mut file = tokio::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.deleted_log_path)
            .await
            .context("Failed to open deleted_archives log for appending")?;

        let log_line = format!("{}\n", id_to_log);

        file.write_all(log_line.as_bytes())
            .await
            .context("Failed to write to deleted_archives log")?;

        Ok(())
    }
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
struct ArchiveItem {
    #[serde(rename = "ArchiveId")]
    archive_id: String,
}

#[derive(Deserialize, Debug)]
struct Inventory {
    #[serde(rename = "ArchiveList")]
    archive_list: HashSet<ArchiveItem>,
}

impl Inventory {
    async fn new() -> Result<Self> {
        let file_path = env::var("INVENTORY_PATH").context("INVENTORY_PATH must be set")?;

        let file_contents = tokio::fs::read(file_path).await?;
        let inventory: Inventory = serde_json::from_slice(&file_contents)?;
        Ok(inventory)
    }

    pub fn get_id_set(&self) -> HashSet<String> {
        self.archive_list
            .iter()
            .map(|item| item.archive_id.clone())
            .collect()
    }
}

#[derive(Deserialize, Debug)]
struct DeletedInventory {
    ids: HashSet<String>,
}

impl DeletedInventory {
    pub async fn new(file_path: &str) -> Result<Self> {
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
