use crate::file_service::FileService;
use anyhow::{Context, Result};
use aws_config::{BehaviorVersion, Region, meta::region::RegionProviderChain};
use aws_sdk_glacier::Client as GlacierClient;
use futures::stream::{self, StreamExt};
use std::{env, sync::Arc};

pub struct AwsService {
    client: GlacierClient,
}

impl AwsService {
    pub async fn new() -> Result<Self> {
        let region = env::var("REGION").context("REGION must be set")?;
        let profile = env::var("PROFILE").context("PROFILE must be set")?;

        let region_provider = RegionProviderChain::default_provider().or_else(Region::new(region));

        let config = aws_config::defaults(BehaviorVersion::latest())
            .region(region_provider)
            .profile_name(profile)
            .load()
            .await;
        let client = GlacierClient::new(&config);
        Ok(Self { client })
    }

    pub async fn delete_archives(&self, file_service: FileService) -> Result<()> {
        let vault_name = env::var("VAULT_NAME").context("VAULT_NAME must be set")?;
        let parallel_jobs: usize = env::var("PARALLEL_JOBS")
            .context("PARALLEL_JOBS must be set")?
            .parse()
            .context("PARALLEL_JOBS must be a valid number")?;

        println!(
            "Starting deletion for {} archives with a concurrency of {}.",
            file_service.delete_ids.len(),
            parallel_jobs
        );

        let file_service_arc = Arc::new(file_service);

        let stream = stream::iter(&file_service_arc.delete_ids)
            .map(|id| {
                let client = self.client.clone();
                let vault_name = vault_name.clone();
                let id = id.clone();
                tokio::spawn(async move { delete_one_archive(&client, &vault_name, id).await })
            })
            .buffer_unordered(parallel_jobs);

        stream
            .for_each_concurrent(None, |result| {
                let fs_clone = Arc::clone(&file_service_arc);
                async move {
                    match result {
                        Ok(Ok(archive_id)) => {
                            println!("✅ Success: Deleted {}", archive_id);
                            if let Err(e) = fs_clone.log_deleted_id(&archive_id).await {
                                eprintln!("🔥 Failed to log deleted ID {}: {}", archive_id, e);
                            }
                        }
                        Ok(Err(sdk_error)) => {
                            eprintln!("❌ AWS SDK Error: {}", sdk_error);
                        }
                        Err(join_error) => {
                            eprintln!("🔥 Tokio Task Error: {}", join_error);
                        }
                    }
                }
            })
            .await;

        println!("--- Deletion process complete. ---");
        Ok(())
    }
}

async fn delete_one_archive(
    client: &GlacierClient,
    vault_name: &str,
    archive_id: String,
) -> Result<String, aws_sdk_glacier::Error> {
    client
        .delete_archive()
        .vault_name(vault_name)
        .archive_id(&archive_id)
        .send()
        .await?;

    Ok(archive_id)
}
