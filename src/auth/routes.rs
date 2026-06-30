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
        session::{clear_user_cookie, set_user_cookie, user_id_from_jar},
    },
    error::{render, AppError, AppResult},
    state::AppState,
};

#[derive(Template)]
#[template(path = "login.html")]
struct LoginTemplate {
    username: Option<String>,
    mode: &'static str,
    error: Option<String>,
}

#[derive(Deserialize)]
struct AuthForm {
    username: String,
    password: String,
}

#[derive(Deserialize)]
struct ErrorQuery {
    error: Option<String>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(home))
        .route("/login", get(login_page).post(login))
        .route("/register", get(register_page).post(register))
        .route("/logout", post(logout))
}

async fn home(jar: PrivateCookieJar) -> Response {
    if user_id_from_jar(&jar).is_some() {
        Redirect::to("/pantry").into_response()
    } else {
        Redirect::to("/login").into_response()
    }
}

async fn login_page(Query(query): Query<ErrorQuery>) -> AppResult<Response> {
    render(LoginTemplate {
        username: None,
        mode: "login",
        error: query.error,
    })
}

async fn register_page(Query(query): Query<ErrorQuery>) -> AppResult<Response> {
    render(LoginTemplate {
        username: None,
        mode: "register",
        error: query.error,
    })
}

async fn login(
    State(state): State<AppState>,
    jar: PrivateCookieJar,
    Form(form): Form<AuthForm>,
) -> Result<Response, AppError> {
    let username = form.username.trim();
    let password = form.password.trim();

    if username.is_empty() || password.is_empty() {
        return Ok(Redirect::to("/login?error=Username+and+password+are+required").into_response());
    }

    let user = sqlx::query_as::<_, UserRow>(
        "SELECT id, password_hash FROM users WHERE username = ?",
    )
    .bind(username)
    .fetch_optional(&state.db)
    .await?;

    let Some(user) = user else {
        return Ok(Redirect::to("/login?error=Invalid+username+or+password").into_response());
    };

    if !verify_password(password, &user.password_hash) {
        return Ok(Redirect::to("/login?error=Invalid+username+or+password").into_response());
    }

    let jar = set_user_cookie(jar, user.id);
    Ok((jar, Redirect::to("/pantry")).into_response())
}

async fn register(
    State(state): State<AppState>,
    jar: PrivateCookieJar,
    Form(form): Form<AuthForm>,
) -> Result<Response, AppError> {
    let username = form.username.trim();
    let password = form.password.trim();

    if username.len() < 3 {
        return Ok(
            Redirect::to("/register?error=Username+must+be+at+least+3+characters").into_response(),
        );
    }

    if password.len() < 6 {
        return Ok(
            Redirect::to("/register?error=Password+must+be+at+least+6+characters").into_response(),
        );
    }

    let password_hash = hash_password(password)
        .map_err(|e| AppError::msg(format!("failed to hash password: {e}")))?;

    let result = sqlx::query("INSERT INTO users (username, password_hash) VALUES (?, ?)")
        .bind(username)
        .bind(password_hash)
        .execute(&state.db)
        .await;

    match result {
        Ok(row) => {
            let jar = set_user_cookie(jar, row.last_insert_rowid());
            Ok((jar, Redirect::to("/pantry")).into_response())
        }
        Err(sqlx::Error::Database(db_err)) if db_err.is_unique_violation() => Ok(Redirect::to(
            "/register?error=That+username+is+already+taken",
        )
            .into_response()),
        Err(e) => Err(e.into()),
    }
}

async fn logout(jar: PrivateCookieJar) -> Response {
    let jar = clear_user_cookie(jar);
    (jar, Redirect::to("/login")).into_response()
}

pub async fn current_username(state: &AppState, user_id: i64) -> AppResult<String> {
    let row = sqlx::query_scalar::<_, String>("SELECT username FROM users WHERE id = ?")
        .bind(user_id)
        .fetch_one(&state.db)
        .await?;
    Ok(row)
}

#[derive(sqlx::FromRow)]
struct UserRow {
    id: i64,
    password_hash: String,
}
