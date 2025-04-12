pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
  NodejsAlreadyRunning,
  NodejsNotRunning,
}

impl std::fmt::Display for Error {
  fn fmt(
    &self,
    f: &mut std::fmt::Formatter<'_>,
  ) -> std::fmt::Result {
    match self {
      Error::NodejsAlreadyRunning => write!(f, "AlreadyRunning"),
      Error::NodejsNotRunning => write!(f, "NotRunning"),
    }
  }
}

impl std::error::Error for Error {}

impl From<Error> for std::io::Error {
    fn from(value: Error) -> Self {
      std::io::Error::other(value)
    }
}
