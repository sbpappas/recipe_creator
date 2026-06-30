use axum_extra::extract::cookie::{Key, PrivateCookieJar};
use sha2::{Digest, Sha512};

pub const USER_ID_COOKIE: &str = "user_id";

pub fn cookie_key(app_secret: &str) -> Key {
    let digest = Sha512::digest(app_secret.as_bytes());
    Key::from(&digest)
}

pub fn user_id_from_jar(jar: &PrivateCookieJar) -> Option<i64> {
    jar.get(USER_ID_COOKIE)
        .and_then(|cookie| cookie.value().parse().ok())
}

pub fn set_user_cookie(jar: PrivateCookieJar, user_id: i64) -> PrivateCookieJar {
    jar.add(
        axum_extra::extract::cookie::Cookie::build((USER_ID_COOKIE, user_id.to_string()))
            .path("/")
            .http_only(true)
            .same_site(axum_extra::extract::cookie::SameSite::Lax)
            .build(),
    )
}

pub fn clear_user_cookie(jar: PrivateCookieJar) -> PrivateCookieJar {
    jar.remove(
        axum_extra::extract::cookie::Cookie::build(USER_ID_COOKIE)
            .path("/")
            .build(),
    )
}
