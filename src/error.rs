use axum::body::Body;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};
use tracing::error;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug)]
pub enum AppError {
    Unknown(anyhow::Error),
    DomainError(DomainError),
}

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self::Unknown(err.into())
    }
}

impl From<DomainError> for AppError {
    fn from(err: DomainError) -> Self {
        Self::DomainError(err)
    }
}

/// https://www.rfc-editor.org/rfc/rfc9457.html
/// https://www.rfc-editor.org/rfc/rfc7807.html
#[derive(Serialize, Deserialize)]
struct ProblemDetails {
    /// A URN. Namespace is `api-error`.
    pub r#type: &'static str,
    pub title: String,
    pub status: u16,
    pub detail: Option<String>,
    pub instance: Option<String>,
}

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for AppError {
    fn into_response(self) -> Response<Body> {
        if let AppError::Unknown(err) = &self {
            error!("Internal error: {}", err);
        }

        let mut headers = HeaderMap::new();
        headers.insert("Content-Type", "application/problem+json".parse().unwrap());

        let code = match &self {
            Self::Unknown(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::DomainError(e) => e.status(),
        };
        let problem_details = ProblemDetails {
            r#type: match &self {
                Self::Unknown(_) => "urn:api-error:common.internal",
                Self::DomainError(e) => e.code(),
            },
            title: match &self {
                Self::Unknown(e) => e.to_string(),
                Self::DomainError(e) => e.description().to_owned(),
            },
            status: code.as_u16(),
            detail: None,
            instance: None,
        };

        (code, headers, Json(problem_details)).into_response()
    }
}

macro_rules! _eliminate_fields {
    ($name: ident {$($field: ident),+ $(,)?}) => {
        DomainError::$name { .. }
    };
    ($name: ident) => {
        DomainError::$name
    };
}

/// https://datatracker.ietf.org/doc/html/rfc7807#section-3
macro_rules! domain_errors {
    ( $($name: ident $({$($field: ident: $type: ty),+ $(,)?})?, $code: literal, $status: expr, $description: literal;)+ ) => {
        #[derive(Debug)]
        pub enum DomainError {
            $(#[allow(unused)] #[doc = $description] $name $({$($field: $type),+})?),+
        }

        impl DomainError {
            pub fn code(&self) -> &'static str {
                match self {
                    $(_eliminate_fields!($name $({$($field)+})?) => concat!("urn:api-error:", $code),)+
                }
            }

            pub fn status(&self) -> StatusCode {
                match self {
                    $(_eliminate_fields!($name $({$($field)+})?) => $status,)+
                }
            }

            pub fn description(&self) -> &'static str {
                match self {
                    $(_eliminate_fields!($name $({$($field)+})?) => $description,)+
                }
            }
        }
    };
}

domain_errors! {
    Foo, "common.foo", StatusCode::NOT_FOUND, "description";
    Bar, "common.bar", StatusCode::BAD_REQUEST, "description";
    User {user_id: usize}, "user.error", StatusCode::NOT_FOUND, "description";
}
