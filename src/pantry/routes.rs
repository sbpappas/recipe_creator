use askama::Template;
use axum::{
    extract::{Form, State},
    response::{IntoResponse, Redirect, Response},
    routing::{get, post},
    Router,
};
use axum_extra::extract::PrivateCookieJar;
use serde::Deserialize;

use crate::{
    auth::{
        routes::current_username,
        session::user_id_from_jar,
    },
    error::{render, AppError, AppResult},
    state::AppState,
};

#[derive(Template)]
#[template(path = "pantry.html")]
struct PantryTemplate {
    username: Option<String>,
    items: Vec<String>,
    error: Option<String>,
}

#[derive(Deserialize)]
struct AddItemForm {
    name: String,
}

#[derive(Deserialize)]
struct DeleteItemForm {
    name: String,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/pantry", get(pantry_page).post(add_item))
        .route("/pantry/delete", post(delete_item))
}

async fn pantry_page(
    State(state): State<AppState>,
    jar: PrivateCookieJar,
) -> AppResult<Response> {
    let user_id = user_id(&jar)?;
    let username = current_username(&state, user_id).await?;
    let items = list_pantry(&state, user_id).await?;

    render(PantryTemplate {
        username: Some(username),
        items,
        error: None,
    })
}

async fn add_item(
    State(state): State<AppState>,
    jar: PrivateCookieJar,
    Form(form): Form<AddItemForm>,
) -> Result<Response, AppError> {
    let user_id = user_id(&jar)?;
    let name = form.name.trim();

    if name.is_empty() {
        return Ok(Redirect::to("/pantry").into_response());
    }

    let result = sqlx::query("INSERT INTO pantry_items (user_id, name) VALUES (?, ?)")
        .bind(user_id)
        .bind(name)
        .execute(&state.db)
        .await;

    match result {
        Ok(_) => Ok(Redirect::to("/pantry").into_response()),
        Err(sqlx::Error::Database(db_err)) if db_err.is_unique_violation() => {
            Ok(Redirect::to("/pantry").into_response())
        }
        Err(e) => Err(e.into()),
    }
}

async fn delete_item(
    State(state): State<AppState>,
    jar: PrivateCookieJar,
    Form(form): Form<DeleteItemForm>,
) -> AppResult<Response> {
    let user_id = user_id(&jar)?;
    sqlx::query("DELETE FROM pantry_items WHERE user_id = ? AND name = ?")
        .bind(user_id)
        .bind(form.name.trim())
        .execute(&state.db)
        .await?;
    Ok(Redirect::to("/pantry").into_response())
}

fn user_id(jar: &PrivateCookieJar) -> AppResult<i64> {
    user_id_from_jar(jar).ok_or(AppError::Unauthorized)
}

async fn list_pantry(state: &AppState, user_id: i64) -> AppResult<Vec<String>> {
    let rows = sqlx::query_scalar::<_, String>(
        "SELECT name FROM pantry_items WHERE user_id = ? ORDER BY name COLLATE NOCASE",
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await?;
    Ok(rows)
}

pub async fn pantry_for_user(state: &AppState, user_id: i64) -> AppResult<Vec<String>> {
    list_pantry(state, user_id).await
}
