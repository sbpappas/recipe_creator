use askama::Template;
use axum::{
    extract::{Form, Path, Query, State},
    response::{IntoResponse, Redirect, Response},
    routing::get,
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
    llm::{GeminiClient, GeneratedRecipe},
    pantry::routes::pantry_for_user,
    recipes::prompt::build_request,
    state::AppState,
};

#[derive(Template)]
#[template(path = "generate.html")]
struct GenerateTemplate {
    username: Option<String>,
    pantry_count: usize,
    gemini_configured: bool,
    error: Option<String>,
}

#[derive(Template)]
#[template(path = "recipe.html")]
struct RecipeTemplate {
    username: Option<String>,
    recipe: GeneratedRecipe,
    meal_type: String,
}

#[derive(Template)]
#[template(path = "history.html")]
struct HistoryTemplate {
    username: Option<String>,
    recipes: Vec<HistoryItem>,
}

#[derive(Clone)]
pub struct HistoryItem {
    pub id: i64,
    pub title: String,
    pub meal_type: String,
    pub created_at: String,
}

#[derive(Deserialize)]
pub struct GenerateForm {
    meal_type: String,
    must_use: String,
    notes: String,
}

#[derive(Deserialize)]
struct ErrorQuery {
    error: Option<String>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/recipes/new", get(generate_page).post(generate_recipe))
        .route("/recipes/history", get(history_page))
        .route("/recipes/:id", get(show_recipe))
}

async fn generate_page(
    State(state): State<AppState>,
    jar: PrivateCookieJar,
    Query(query): Query<ErrorQuery>,
) -> AppResult<Response> {
    let user_id = user_id(&jar)?;
    let username = current_username(&state, user_id).await?;
    let pantry = pantry_for_user(&state, user_id).await?;

    render(GenerateTemplate {
        username: Some(username),
        pantry_count: pantry.len(),
        gemini_configured: state.config.gemini_api_key.is_some(),
        error: query.error,
    })
}

async fn generate_recipe(
    State(state): State<AppState>,
    jar: PrivateCookieJar,
    Form(form): Form<GenerateForm>,
) -> Result<Response, AppError> {
    let user_id = user_id(&jar)?;

    let Some(api_key) = state.config.gemini_api_key.clone() else {
        return Ok(Redirect::to("/recipes/new?error=GEMINI_API_KEY+is+not+configured").into_response());
    };

    let pantry = pantry_for_user(&state, user_id).await?;
    let request = build_request(
        &form.meal_type,
        pantry,
        &form.must_use,
        Some(form.notes),
    );

    let client = GeminiClient::new(
        state.http.clone(),
        api_key,
        state.config.gemini_model.clone(),
    );

    let recipe = match client.generate_recipe(&request).await {
        Ok(recipe) => recipe,
        Err(e) => {
            let message = e.to_string();
            let encoded = urlencoding::encode(&message);
            return Ok(Redirect::to(&format!("/recipes/new?error={encoded}")).into_response());
        }
    };

    let prompt_snapshot = serde_json::to_string(&request)
        .map_err(|e| AppError::msg(format!("failed to serialize prompt: {e}")))?;
    let recipe_json = serde_json::to_string(&recipe)
        .map_err(|e| AppError::msg(format!("failed to serialize recipe: {e}")))?;

    let result = sqlx::query(
        "INSERT INTO recipes (user_id, title, meal_type, prompt_snapshot, recipe_json) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(user_id)
    .bind(&recipe.title)
    .bind(&form.meal_type)
    .bind(prompt_snapshot)
    .bind(recipe_json)
    .execute(&state.db)
    .await?;

    Ok(Redirect::to(&format!("/recipes/{}", result.last_insert_rowid())).into_response())
}

async fn show_recipe(
    State(state): State<AppState>,
    jar: PrivateCookieJar,
    Path(id): Path<i64>,
) -> AppResult<Response> {
    let user_id = user_id(&jar)?;
    let username = current_username(&state, user_id).await?;

    let row = sqlx::query_as::<_, RecipeRow>(
        "SELECT meal_type, recipe_json FROM recipes WHERE id = ? AND user_id = ?",
    )
    .bind(id)
    .bind(user_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound)?;

    let recipe: GeneratedRecipe = serde_json::from_str(&row.recipe_json)
        .map_err(|e| AppError::msg(format!("stored recipe is invalid: {e}")))?;

    render(RecipeTemplate {
        username: Some(username),
        recipe,
        meal_type: row.meal_type,
    })
}

async fn history_page(
    State(state): State<AppState>,
    jar: PrivateCookieJar,
) -> AppResult<Response> {
    let user_id = user_id(&jar)?;
    let username = current_username(&state, user_id).await?;

    let rows = sqlx::query_as::<_, HistoryRow>(
        "SELECT id, title, meal_type, created_at FROM recipes WHERE user_id = ? ORDER BY created_at DESC",
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await?;

    let recipes = rows
        .into_iter()
        .map(|row| HistoryItem {
            id: row.id,
            title: row.title,
            meal_type: row.meal_type,
            created_at: row.created_at,
        })
        .collect();

    render(HistoryTemplate {
        username: Some(username),
        recipes,
    })
}

fn user_id(jar: &PrivateCookieJar) -> AppResult<i64> {
    user_id_from_jar(jar).ok_or(AppError::Unauthorized)
}

#[derive(sqlx::FromRow)]
struct RecipeRow {
    meal_type: String,
    recipe_json: String,
}

#[derive(sqlx::FromRow)]
struct HistoryRow {
    id: i64,
    title: String,
    meal_type: String,
    created_at: String,
}
