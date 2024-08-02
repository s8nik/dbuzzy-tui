use dbuzzy::db::{ConnectionConfig, PgPool};
use deadpool_postgres::GenericClient;
use testcontainers::{
    core::{IntoContainerPort, WaitFor},
    runners::AsyncRunner,
    ContainerAsync, GenericImage, ImageExt,
};

const NAME: &str = "postgres";
const TAG: &str = "16-alpine";

pub async fn setup() -> anyhow::Result<(ContainerAsync<GenericImage>, PgPool)> {
    let image = GenericImage::new(NAME, TAG)
        .with_wait_for(WaitFor::message_on_stderr(
            "database system is ready to accept connections",
        ))
        .with_env_var("POSTGRES_USER", "postgres")
        .with_env_var("POSTGRES_DB", "postgres")
        .with_env_var("POSTGRES_PASSWORD", "postgres")
        .with_mapped_port(5432, 5432.tcp());

    let container = image.start().await?;

    let config = ConnectionConfig {
        name: None,
        host: "127.0.0.1".to_owned(),
        port: 5432,
        user: "postgres".to_owned(),
        dbname: Some("postgres".to_owned()),
        password: Some("postgres".to_owned()),
    };

    let pool = PgPool::create(&config)?;

    let connection = pool.acquire().await?;

    let dbname = "test";

    let does_exist: bool = connection
        .query_one(
            "SELECT EXISTS(SELECT 1 FROM pg_catalog.pg_database WHERE datname = $1)",
            &[&dbname],
        )
        .await?
        .get(0);

    if !does_exist {
        connection
            .execute("SELECT 'CREATE DATABASE test'", &[])
            .await?;
    }

    connection
        .execute("CREATE SCHEMA IF NOT EXISTS foo;", &[])
        .await?;

    connection
        .execute(
            r#"
                CREATE TABLE IF NOT EXISTS foo.bar (
                    id INTEGER,
                    baz varchar(50)
                );
            "#,
            &[],
        )
        .await?;

    Ok((container, pool))
}
