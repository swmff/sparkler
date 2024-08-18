use askama_axum::Template;
use axum::response::IntoResponse;
use axum::{extract::State, response::Html};
use axum_extra::extract::CookieJar;

use xsu_authman::model::Profile;

use crate::config::Config;
use crate::database::Database;
use crate::model::DatabaseError;

use super::clean_metadata;

#[derive(Template)]
#[template(path = "settings/account.html")]
struct AccountSettingsTemplate {
    config: Config,
    profile: Option<Profile>,
    unread: usize,
    notifs: usize,
    metadata: String,
}

/// GET /settings
pub async fn account_settings(
    jar: CookieJar,
    State(database): State<Database>,
) -> impl IntoResponse {
    let auth_user = match jar.get("__Secure-Token") {
        Some(c) => match database
            .auth
            .get_profile_by_unhashed(c.value_trimmed().to_string())
            .await
        {
            Ok(ua) => ua,
            Err(_) => return Html(DatabaseError::NotAllowed.to_html(database)),
        },
        None => return Html(DatabaseError::NotAllowed.to_html(database)),
    };

    let unread = match database
        .get_questions_by_recipient(auth_user.id.to_owned())
        .await
    {
        Ok(unread) => unread.len(),
        Err(_) => 0,
    };

    let notifs = database
        .auth
        .get_notification_count_by_recipient(auth_user.id.to_owned())
        .await;

    Html(
        AccountSettingsTemplate {
            config: database.server_options,
            metadata: clean_metadata(&auth_user.metadata),
            profile: Some(auth_user),
            unread,
            notifs,
        }
        .render()
        .unwrap(),
    )
}

#[derive(Template)]
#[template(path = "settings/profile.html")]
struct ProfileSettingsTemplate {
    config: Config,
    profile: Option<Profile>,
    unread: usize,
    notifs: usize,
    metadata: String,
}

/// GET /settings/profile
pub async fn profile_settings(
    jar: CookieJar,
    State(database): State<Database>,
) -> impl IntoResponse {
    let auth_user = match jar.get("__Secure-Token") {
        Some(c) => match database
            .auth
            .get_profile_by_unhashed(c.value_trimmed().to_string())
            .await
        {
            Ok(ua) => ua,
            Err(_) => return Html(DatabaseError::NotAllowed.to_html(database)),
        },
        None => return Html(DatabaseError::NotAllowed.to_html(database)),
    };

    let unread = match database
        .get_questions_by_recipient(auth_user.id.to_owned())
        .await
    {
        Ok(unread) => unread.len(),
        Err(_) => 0,
    };

    let notifs = database
        .auth
        .get_notification_count_by_recipient(auth_user.id.to_owned())
        .await;

    Html(
        ProfileSettingsTemplate {
            config: database.server_options,
            metadata: clean_metadata(&auth_user.metadata),
            profile: Some(auth_user),
            unread,
            notifs,
        }
        .render()
        .unwrap(),
    )
}

#[derive(Template)]
#[template(path = "settings/privacy.html")]
struct PrivacySettingsTemplate {
    config: Config,
    profile: Option<Profile>,
    unread: usize,
    notifs: usize,
    metadata: String,
}

/// GET /settings/privacy
pub async fn privacy_settings(
    jar: CookieJar,
    State(database): State<Database>,
) -> impl IntoResponse {
    let auth_user = match jar.get("__Secure-Token") {
        Some(c) => match database
            .auth
            .get_profile_by_unhashed(c.value_trimmed().to_string())
            .await
        {
            Ok(ua) => ua,
            Err(_) => return Html(DatabaseError::NotAllowed.to_html(database)),
        },
        None => return Html(DatabaseError::NotAllowed.to_html(database)),
    };

    let unread = match database
        .get_questions_by_recipient(auth_user.id.to_owned())
        .await
    {
        Ok(unread) => unread.len(),
        Err(_) => 0,
    };

    let notifs = database
        .auth
        .get_notification_count_by_recipient(auth_user.id.to_owned())
        .await;

    Html(
        PrivacySettingsTemplate {
            config: database.server_options,
            metadata: clean_metadata(&auth_user.metadata),
            profile: Some(auth_user),
            unread,
            notifs,
        }
        .render()
        .unwrap(),
    )
}
