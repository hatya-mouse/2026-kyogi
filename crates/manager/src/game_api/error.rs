use std::fmt::Display;

#[derive(Debug)]
pub(crate) enum ApiError {
    Transport(reqwest::Error),
    Http {
        status: reqwest::StatusCode,
        body: String,
    },
}

impl Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Transport(err) => write!(f, "{}", err),
            Self::Http { status, body } => write!(f, "{}: {}", status, body),
        }
    }
}
