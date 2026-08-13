use std::{fmt, io};

/// Errors that can occur while initializing tracing subscriber.
#[derive(Debug)]
pub enum TracingError {
    File(io::Error),
    Filter(tracing_subscriber::filter::ParseError),
    Subscriber(tracing::subscriber::SetGlobalDefaultError),
    TryInit(tracing_subscriber::util::TryInitError),
}

impl fmt::Display for TracingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TracingError::File(error) => write!(f, "failed to open log file: {error}"),
            TracingError::Filter(error) => {
                write!(f, "invalid filter directive: {error}")
            }
            TracingError::Subscriber(error) => {
                write!(f, "tracing subscriber already set: {error}")
            }
            TracingError::TryInit(error) => {
                write!(f, "tracing subscriber initialization: {error}")
            }
        }
    }
}

impl std::error::Error for TracingError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            TracingError::File(source) => Some(source),
            TracingError::Filter(source) => Some(source),
            TracingError::Subscriber(source) => Some(source),
            TracingError::TryInit(source) => Some(source),
        }
    }
}

impl From<io::Error> for TracingError {
    fn from(source: io::Error) -> Self {
        TracingError::File(source)
    }
}

impl From<tracing_subscriber::filter::ParseError> for TracingError {
    fn from(source: tracing_subscriber::filter::ParseError) -> Self {
        TracingError::Filter(source)
    }
}

impl From<tracing::subscriber::SetGlobalDefaultError> for TracingError {
    fn from(source: tracing::subscriber::SetGlobalDefaultError) -> Self {
        TracingError::Subscriber(source)
    }
}

impl From<tracing_subscriber::util::TryInitError> for TracingError {
    fn from(source: tracing_subscriber::util::TryInitError) -> Self {
        TracingError::TryInit(source)
    }
}
