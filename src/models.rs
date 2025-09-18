use serde::Deserialize;
use std::collections::HashSet;

#[derive(Deserialize, Debug)]
struct ArchiveItem {
    #[serde(rename = "ArchiveId")]
    archive_id: String,
}

#[derive(Deserialize, Debug)]
struct Inventory {
    #[serde(rename = "ArchiveList")]
    archive_list: Vec<ArchiveItem>,
}

impl Inventory {
    pub fn create_id_hashset(&self) -> HashSet<String> {
        self.archive_list
            .iter()
            .map(|item| item.archive_id.clone())
            .collect()
    }
}
