use anyhow::Result;
use car_mirror::{cache::NoCache, common::Config};
use reqwest::Client;
use car_mirror_reqwest::RequestBuilderExt;
use wnfs_common::{BlockStore, MemoryBlockStore, CODEC_RAW};

#[tokio::main]
async fn main() -> Result<()> {
    // Start a car-mirror axum webserver:
    tokio::spawn(car_mirror_axum::serve(MemoryBlockStore::new()));

    // Generate some test IPLD data:
    let store = MemoryBlockStore::new();
    let data = b"Hello, world!".to_vec();
    let root = store.put_block(data, CODEC_RAW).await?;

    // Run the car mirror push protocol to upload the IPLD data:
    let client = Client::new();
    client
        .post(format!("http://localhost:3344/dag/push/{root}"))
        .run_car_mirror_push(root, &store, &NoCache)
        .await?;

    let store = MemoryBlockStore::new(); // clear out data
    // Download the same data again:
    client
        .post(format!("http://localhost:3344/dag/pull/{root}"))
        .run_car_mirror_pull(root, &Config::default(), &store, &NoCache)
        .await?;

    assert!(store.has_block(&root).await?);
    Ok(())
}