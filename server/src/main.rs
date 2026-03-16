#[tokio::main]
async fn main() -> anyhow::Result<()> {
    hash_server::run().await
}
