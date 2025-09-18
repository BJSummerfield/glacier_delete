mod aws_service;
mod file_service;
use anyhow::Result;
use aws_service::AwsService;
use dotenvy::dotenv;
use file_service::FileService;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    let file_service_fut = FileService::new();
    let aws_service_fut = AwsService::new();
    let (file_service, aws_service) = tokio::try_join!(file_service_fut, aws_service_fut)?;

    aws_service.delete_archives(file_service).await?;
    Ok(())
}
