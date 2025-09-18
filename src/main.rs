mod inventory;
use anyhow::Result;
use dotenvy::dotenv;
use inventory::{DeletedInventory, Inventory};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    let mut inventory = Inventory::new().await?;
    let deleted_inventory = DeletedInventory::new().await?;
    inventory.filter_deleted(&deleted_inventory.ids);
    println!("{:?}", inventory.archive_list.len());
    Ok(())
}
