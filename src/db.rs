use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};

pub async fn connect(database_url: &str) -> Result<SqlitePool, sqlx::Error> {
    if let Some(parent) = database_url
        .strip_prefix("sqlite:")
        .and_then(|path| path.split('?').next())
        .and_then(|path| std::path::Path::new(path).parent())
    {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).ok();
        }
    }

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}
