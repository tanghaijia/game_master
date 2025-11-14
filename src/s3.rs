use std::env;
use aws_sdk_s3::Client;
use aws_sdk_s3::config::{BehaviorVersion, Credentials, Region};
use aws_sdk_s3::primitives::ByteStream;
use tokio::fs;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

pub struct Config {
    pub region: String,
    pub access_key_id: String,
    pub secret_access_key: String,
    pub endpoint_url: String,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        let region = env::var("RUSTFS_REGION")?;
        let access_key_id = env::var("RUSTFS_ACCESS_KEY_ID")?;
        let secret_access_key = env::var("RUSTFS_SECRET_ACCESS_KEY")?;
        let endpoint_url = env::var("RUSTFS_ENDPOINT_URL")?;

        Ok(Config {
            region,
            access_key_id,
            secret_access_key,
            endpoint_url,
        })
    }
}

pub async fn get_rustfs_client() -> anyhow::Result<Client> {
    let config = Config::from_env()?;

    let credentials = Credentials::new(
        config.access_key_id,
        config.secret_access_key,
        None,
        None,
        "rustfs",
    );

    let region = Region::new(config.region);

    let endpoint_url = config.endpoint_url;

    let shard_config = aws_config::defaults(BehaviorVersion::latest())
        .region(region)
        .credentials_provider(credentials)
        .endpoint_url(endpoint_url)
        .load()
        .await;

    let rustfs_client = Client::new(&shard_config);

    Ok(rustfs_client)
}

pub async fn upload_file(rustfs_client: &Client, path: &str, bucket: &str, key: &str) -> anyhow::Result<()> {
    let data = fs::read(path).await?;

    match rustfs_client
        .put_object()
        .bucket(bucket)
        .key(key)
        .body(ByteStream::from(data))
        .send()
        .await
    {
        Ok(res) => {
            println!("Object uploaded successfully, res: {:?}", res);
            Ok(())
        }
        Err(e) => {
            println!("Error uploading object: {:?}", e);
            return Err(e.into());
        }
    }
}

pub async fn download_file(rustfs_client: &Client, path: &str, bucket: &str, key: &str) -> anyhow::Result<()> {
    let _ = fs::remove_file(path).await;
    let mut file = File::create(path).await?;
    let mut object = rustfs_client.get_object().bucket(bucket).key(key).send().await?;
    while let Some(bytes) = object.body.try_next().await? {
        file.write_all(&bytes).await?;
    }
    println!("downloaded successfully");
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::s3::{download_file, get_rustfs_client};

    #[tokio::test]
    async fn test_rustfs_client() {
        dotenv::dotenv().ok();
        let rustfs_client = get_rustfs_client().await.unwrap();
        let res = rustfs_client.list_buckets().send().await.unwrap();

        println!("Total buckets number is {:?}", res.buckets().len());
        for bucket in res.buckets() {
            println!("Bucket: {:?}", bucket.name());
        }

    }

    #[tokio::test]
    async fn test_download_file() {
        dotenv::dotenv().ok();
        let rustfs_client = get_rustfs_client().await.unwrap();
        download_file(&rustfs_client,"C:\\Users\\89396\\projects\\game_master\\file.zip", "days7server", "000001/MyGame.zip").await.unwrap();

    }
}
