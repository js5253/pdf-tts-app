use std::fmt::Display;

use serde::Serialize;


#[derive(Debug, specta::Type, Clone)]
pub struct CommandError(pub Error);

impl Serialize for CommandError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&format!("{:?}", self.0))
    }
}
pub type CommandResult<T> = std::result::Result<T, CommandError>;

impl Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}
impl<E> From<E> for CommandError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into().into())
    }
}

#[derive(Debug, specta::Type, Serialize, Clone)]
pub struct Error(pub String);

impl From<anyhow::Error> for Error {
    fn from(error: anyhow::Error) -> Self {
        Self(format!("{error:#}"))
    }
}