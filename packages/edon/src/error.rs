use std::sync::Arc;

use crate::internal::constants::LIB_NAME;

pub type Result<T> = std::result::Result<T, Error>;

pub enum Error {
  NodejsAlreadyRunning,
  NodejsNotRunning,
  LibnodeNotLoaded,
  LibnodeNotFound,
  LibnodeFailedToLoad,
  LibnodeSymbolNotFound,
  IoError(Arc<std::io::Error>),
}

impl std::fmt::Debug for Error {
  fn fmt(
    &self,
    f: &mut std::fmt::Formatter<'_>,
  ) -> std::fmt::Result {
    match self {
      Self::NodejsAlreadyRunning => write!(f, "NodejsAlreadyRunning"),
      Self::NodejsNotRunning => write!(f, "NodejsNotRunning"),
      Self::LibnodeNotLoaded => write!(f, "LibnodeNotLoaded"),
      Self::LibnodeNotFound => write!(f, "{}", self),
      Self::LibnodeFailedToLoad => write!(f, "LibnodeFailedToLoad"),
      Self::LibnodeSymbolNotFound => write!(f, "LibnodeSymbolNotFound"),
      Self::IoError(arg0) => f.debug_tuple("IoError").field(arg0).finish(),
    }
  }
}

impl std::fmt::Display for Error {
  fn fmt(
    &self,
    f: &mut std::fmt::Formatter<'_>,
  ) -> std::fmt::Result {
    match self {
      Error::NodejsAlreadyRunning => write!(f, "AlreadyRunning"),
      Error::NodejsNotRunning => write!(f, "NotRunning"),
      Error::LibnodeFailedToLoad => write!(f, "LibnodeFailedToLoad"),
      Error::LibnodeNotLoaded => write!(f, "LibnodeNotLoaded"),
      Error::LibnodeSymbolNotFound => write!(f, "LibnodeSymbolNotFound"),
      Error::IoError(err) => write!(f, "{}", err),
      Error::LibnodeNotFound => write!(
        f,
        r#"NotFound: {}
How to fix:
    - Specify $EDON_LIBNODE_PATH environment variable
    - Place {} in '<executable_path>/{}'
    - Place {} in '<executable_path>/../lib/{}'
"#,
        LIB_NAME, LIB_NAME, LIB_NAME, LIB_NAME, LIB_NAME
      ),
    }
  }
}

impl std::error::Error for Error {}

impl From<&Error> for Error {
  fn from(value: &Error) -> Self {
    match value {
      Error::NodejsAlreadyRunning => Error::NodejsAlreadyRunning,
      Error::NodejsNotRunning => Error::NodejsNotRunning,
      Error::LibnodeNotFound => Error::LibnodeNotFound,
      Error::LibnodeNotLoaded => Error::LibnodeNotLoaded,
      Error::LibnodeFailedToLoad => Error::LibnodeFailedToLoad,
      Error::LibnodeSymbolNotFound => Error::LibnodeSymbolNotFound,
      Error::IoError(error) => Error::IoError(error.clone()),
    }
  }
}

impl From<Error> for std::io::Error {
  fn from(value: Error) -> Self {
    std::io::Error::other(value)
  }
}

impl From<std::io::Error> for Error {
  fn from(value: std::io::Error) -> Self {
    Self::IoError(Arc::new(value))
  }
}