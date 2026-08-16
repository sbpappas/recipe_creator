use std::env;
use argon2::{password_hash::{rand_core::OsRng, PasswordHasher, SaltString}, Argon2};
use sqlx::sqlite::SqlitePoolOptions;
use rpassword;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 || args.len() > 3 {
        eprintln!("Usage: reset_password <username> [new-password]");
        std::process::exit(2);
    }

    let username = &args[1];

    // If a password was provided as an arg, use it; otherwise prompt securely
    let new_password: String = if args.len() == 3 {
        args[2].clone()
    } else {
        let p1 = rpassword::prompt_password("New password: ")
            .map_err(|e| -> Box<dyn std::error::Error> { Box::from(e.to_string()) })?;
        let p2 = rpassword::prompt_password("Confirm new password: ")
            .map_err(|e| -> Box<dyn std::error::Error> { Box::from(e.to_string()) })?;
        if p1 != p2 {
            eprintln!("Passwords do not match");
            std::process::exit(2);
        }
        p1
    };

    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:data/recipe_creator.db?mode=rwc".into());

    // Create parent dir if needed (same logic as the app)
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
        .connect(&database_url)
        .await?;

    // Run migrations to ensure schema exists (same as app)
    sqlx::migrate!("./migrations").run(&pool).await?;

    // Hash the new password using Argon2 (same algorithm as the app)
    let salt = SaltString::generate(&mut OsRng);
    let password_hash = Argon2::default()
        .hash_password(new_password.as_bytes(), &salt)
        .map_err(|e| -> Box<dyn std::error::Error> { Box::from(format!("argon2 error: {e}")) })?
        .to_string();

    let result = sqlx::query("UPDATE users SET password_hash = ? WHERE username = ?")
        .bind(password_hash)
        .bind(username)
        .execute(&pool)
        .await?;

    if result.rows_affected() == 0 {
        eprintln!("No user found with username '{username}'");
        std::process::exit(1);
    }

    println!("Password for '{username}' updated successfully.");
    Ok(())
}
