// account page where a logged-in user can change their own password

use askama::Template;
use axum::{
    extract::{Form, Query, State},
    response::{IntoResponse, Redirect, Response},
    routing::{get, post},
    Router,
};
use axum_extra::extract::PrivateCookieJar;
use serde::Deserialize;

use crate::{
    auth::{
        password::{hash_password, verify_password},
        routes::current_username,
        session::user_id_from_jar,
    },
    error::{render, AppError, AppResult},
    i18n::{Lang, Strings},
    state::AppState,
};

#[derive(Template)]
#[template(path = "account.html")]
struct AccountTemplate {
    t: &'static Strings,
    username: Option<String>,
    error: Option<String>,
    password_changed: bool,
}

#[derive(Deserialize)]
struct AccountQuery {
    error: Option<String>,
    status: Option<String>,
}

#[derive(Deserialize)]
struct ChangePasswordForm {
    current_password: String,
    new_password: String,
    confirm_password: String,
}

/// Routes that require login; merged into the protected router in main.rs.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/account", get(account_page))
        .route("/account/password", post(change_password))
}

async fn account_page(
    State(state): State<AppState>,
    jar: PrivateCookieJar,
    Query(query): Query<AccountQuery>,
) -> AppResult<Response> {
    let user_id = user_id_from_jar(&jar).ok_or(AppError::Unauthorized)?;
    let username = current_username(&state, user_id).await?;

    render(AccountTemplate {
        t: Lang::from_jar(&jar).t(),
        username: Some(username),
        error: query.error,
        password_changed: query.status.as_deref() == Some("password_changed"),
    })
}

async fn change_password(
    State(state): State<AppState>,
    jar: PrivateCookieJar,
    Form(form): Form<ChangePasswordForm>,
) -> AppResult<Response> {
    let user_id = user_id_from_jar(&jar).ok_or(AppError::Unauthorized)?;
    // Trimmed the same way as register/login so the new password works at login.
    let current = form.current_password.trim();
    let new = form.new_password.trim();

    let stored_hash = sqlx::query_scalar::<_, String>("SELECT password_hash FROM users WHERE id = ?")
        .bind(user_id)
        .fetch_one(&state.db)
        .await?;

    let error = if !verify_password(current, &stored_hash) {
        Some("wrong_current_password")
    } else if new.len() < 6 {
        Some("password_too_short")
    } else if new != form.confirm_password.trim() {
        Some("passwords_dont_match")
    } else {
        None
    };
    if let Some(code) = error {
        return Ok(Redirect::to(&format!("/account?error={code}")).into_response());
    }

    let new_hash = hash_password(new)
        .map_err(|e| AppError::msg(format!("failed to hash password: {e}")))?;
    sqlx::query("UPDATE users SET password_hash = ? WHERE id = ?")
        .bind(new_hash)
        .bind(user_id)
        .execute(&state.db)
        .await?;

    Ok(Redirect::to("/account?status=password_changed").into_response())
}
