use num_enum::TryFromPrimitiveError;

#[derive(Debug)]
pub struct Error(pub String);

pub type Result<T> = std::result::Result<T, Error>;

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl<E: num_enum::TryFromPrimitive> From<TryFromPrimitiveError<E>> for Error {
    fn from(e: TryFromPrimitiveError<E>) -> Self {
        Self(e.to_string())
    }
}
