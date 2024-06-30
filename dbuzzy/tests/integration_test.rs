mod db;

#[tokio::test]
async fn test_db() -> anyhow::Result<()> {
    let (container, pool) = db::setup().await?;

    let connection = pool.acquire().await?;
    let tree = dbuzzy::db::queries::database_tree(&connection).await?;

    dbg!(tree);

    container.stop().await?;
    Ok(())
}
