use axum::{
    extract::{Request, State},
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::PrivateCookieJar;

use crate::{
    auth::session::user_id_from_jar,
    error::AppError,
    state::AppState,
};

pub async fn require_login(
    State(_state): State<AppState>,
    jar: PrivateCookieJar,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    if user_id_from_jar(&jar).is_none() {
        return Ok(Redirect::to("/login").into_response());
    }

    Ok(next.run(request).await)
}
