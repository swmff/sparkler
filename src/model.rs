use std::collections::HashMap;

use askama_axum::Template;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

use hcaptcha::Hcaptcha;
use serde::{Deserialize, Serialize};
use xsu_authman::model::{Profile, ProfileMetadata};
use xsu_dataman::DefaultReturn;

use crate::database::Database;

/// A question structure
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Question {
    /// The author of the question; "anonymous" marks the question as an anonymous question
    pub author: Profile,
    /// The recipient of the question; cannot be anonymous
    pub recipient: Profile,
    /// The content of the question
    pub content: String,
    /// The ID of the question
    pub id: String,
    /// The time this question was asked
    pub timestamp: u128,
}

impl Question {
    pub fn lost(author: String, recipient: String, content: String, timestamp: u128) -> Self {
        Self {
            author: anonymous_profile(author),
            recipient: anonymous_profile(recipient),
            content,
            id: "".to_string(),
            timestamp,
        }
    }

    pub fn unknown() -> Self {
        Self::lost(
            "anonymous".to_string(),
            String::new(),
            "<lost question>".to_string(),
            0,
        )
    }
}

/// A question structure with ID references to profiles instead of the profiles
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RefQuestion {
    /// The author of the question; "anonymous" marks the question as an anonymous question
    pub author: String,
    /// The recipient of the question; cannot be anonymous
    pub recipient: String,
    /// The content of the question
    pub content: String,
    /// The ID of the question
    pub id: String,
    /// The time this question was asked
    pub timestamp: u128,
}

impl From<Question> for RefQuestion {
    fn from(value: Question) -> Self {
        Self {
            author: value.author.id,
            recipient: value.recipient.id,
            content: value.content,
            id: value.id,
            timestamp: value.timestamp,
        }
    }
}

/// A response structure
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct QuestionResponse {
    /// The author of the response; cannot be anonymous
    pub author: Profile,
    /// The ID question this response is replying to
    pub question: String,
    /// The content of the response
    pub content: String,
    /// The ID of the response
    pub id: String,
    /// The time this response was created
    pub timestamp: u128,
    /// The response tags
    pub tags: Vec<String>,
}

/// A comment structure
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ResponseComment {
    /// The author of the comment; cannot be anonymous
    pub author: Profile,
    /// ID of the response this comment is replying to
    pub response: String,
    /// The content of the comment
    pub content: String,
    /// The ID of the comment
    pub id: String,
    /// The time this comment was created
    pub timestamp: u128,
    /// The ID of the comment this comment is replying to
    pub reply: Option<Box<ResponseComment>>,
}

/// A reaction structure
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Reaction {
    /// The reactor of the reaction; cannot be anonymous
    pub user: Profile,
    /// ID of the asset this reaction is on (response, comment, etc.)
    pub asset: String,
    /// The time this reaction was created
    pub timestamp: u128,
}

/// The status of a user's membership in a [`Circle`]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum MembershipStatus {
    /// A user who has received an invite to a circle, but has not yet accepted
    Pending,
    /// An active member of a circle
    Active,
    /// Not pending or an active member
    Inactive,
}

/// The stored version of a user's membership in a [`Circle`]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CircleMembership {
    /// The ID of the user
    pub user: String,
    /// The ID of the circle
    pub circle: String,
    /// The status of the user's membership in the circle
    pub membership: MembershipStatus,
    /// The time the membership was last updated
    pub timestamp: u128,
}

/// A circle structure
///
/// Circles allow you to post global questions to them (recipient `@circle`),
/// as well as define a custom avatar URL, banner URL, and define a custom theme
///
/// Users can also ask a question and send it to the circle's inbox.
/// This question can then be replied to by anybody in the circle.
///
/// Users can be invited to a circle by the circle's owner. Users are added to the `xcircle_memberships`
/// table with a [`MembershipStatus`] of `Pending`. Users can accept through a notification that is sent
/// to their account, which will then change their [`MembershipStatus`] to `Active`.
///
/// Active members can post to the circle through the compose form. Memberships can always be managed
/// by the owner of the circle, who can remove anybody they want from the circle.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Circle {
    /// The name of the circle
    pub name: String,
    /// The ID of the circle
    pub id: String,
    /// The owner of the circle
    pub owner: Profile,
    /// The metadata of the circle
    pub metadata: CircleMetadata,
    /// The time the circle was created
    pub timestamp: u128,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CircleMetadata {
    pub kv: HashMap<String, String>,
}

// ...

/// Global user profile
pub fn global_profile() -> Profile {
    Profile {
        username: "@".to_string(),
        id: "@".to_string(),
        password: String::new(),
        salt: String::new(),
        tokens: Vec::new(),
        group: 0,
        joined: 0,
        metadata: ProfileMetadata::default(),
    }
}

/// Anonymous user profile
pub fn anonymous_profile(tag: String) -> Profile {
    Profile {
        username: "anonymous".to_string(),
        id: tag,
        password: String::new(),
        salt: String::new(),
        tokens: Vec::new(),
        group: 0,
        joined: 0,
        metadata: ProfileMetadata::default(),
    }
}

// props

#[derive(Serialize, Deserialize, Debug)]
pub struct QuestionCreate {
    pub recipient: String,
    pub content: String,
    pub anonymous: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ResponseCreate {
    pub question: String,
    pub content: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ResponseEdit {
    pub content: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ResponseEditTags {
    pub tags: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CommentCreate {
    pub response: String,
    pub content: String,
    #[serde(default)]
    pub reply: String,
}

#[derive(Serialize, Deserialize, Debug, Hcaptcha)]
pub struct CircleCreate {
    pub name: String,
    #[captcha]
    pub token: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EditCircleMetadata {
    pub metadata: CircleMetadata,
}

/// General API errors
#[derive(Debug)]
pub enum DatabaseError {
    ContentTooShort,
    ContentTooLong,
    InvalidName,
    NotAllowed,
    ValueError,
    OutOfTime,
    NotFound,
    Other,
}

impl DatabaseError {
    pub fn to_string(&self) -> String {
        use DatabaseError::*;
        match self {
            ContentTooShort => String::from("Content too short!"),
            ContentTooLong => String::from("Content too long!"),
            InvalidName => String::from("This name cannot be used!"),
            NotAllowed => String::from("You are not allowed to do this!"),
            ValueError => String::from("One of the field values given is invalid!"),
            OutOfTime => String::from(
                "You can only edit a response within the first 24 hours of posting it!",
            ),
            NotFound => {
                String::from("Nothing with this path exists or you do not have access to it!")
            }
            _ => String::from("An unspecified error has occured"),
        }
    }

    pub fn to_html(&self, database: Database) -> String {
        crate::routing::pages::ErrorTemplate {
            config: database.server_options,
            profile: None,
            message: self.to_string(),
        }
        .render()
        .unwrap()
    }
}

impl IntoResponse for DatabaseError {
    fn into_response(self) -> Response {
        use DatabaseError::*;
        match self {
            NotAllowed => (
                StatusCode::UNAUTHORIZED,
                Json(DefaultReturn::<u16> {
                    success: false,
                    message: self.to_string(),
                    payload: 401,
                }),
            )
                .into_response(),
            NotFound => (
                StatusCode::NOT_FOUND,
                Json(DefaultReturn::<u16> {
                    success: false,
                    message: self.to_string(),
                    payload: 404,
                }),
            )
                .into_response(),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(DefaultReturn::<u16> {
                    success: false,
                    message: self.to_string(),
                    payload: 500,
                }),
            )
                .into_response(),
        }
    }
}

impl<T: Default> Into<xsu_dataman::DefaultReturn<T>> for DatabaseError {
    fn into(self) -> xsu_dataman::DefaultReturn<T> {
        DefaultReturn {
            success: false,
            message: self.to_string(),
            payload: T::default(),
        }
    }
}
