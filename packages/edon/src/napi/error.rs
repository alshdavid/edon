use std::convert::From;
use std::convert::TryFrom;
use std::error;
use std::ffi::CString;
use std::fmt;
#[cfg(feature = "serde-json")]
use std::fmt::Display;
use std::os::raw::c_char;
use std::os::raw::c_void;
use std::ptr;

use libnode_sys;
#[cfg(feature = "serde-json")]
use serde::de;
#[cfg(feature = "serde-json")]
use serde::ser;
#[cfg(feature = "serde-json")]
use serde_json::Error as SerdeJSONError;

use crate::napi::bindgen_runtime::ToNapiValue;
use crate::napi::Env;
use crate::napi::JsUnknown;
use crate::napi::NapiValue;
use crate::napi::Status;

pub type Result<T, S = Status> = std::result::Result<T, Error<S>>;

/// Represent `JsError`.
/// Return this Error in `js_function`, **napi-rs** will throw it as `JsError` for you.
/// If you want throw it as `TypeError` or `RangeError`, you can use `JsTypeError/JsRangeError::from(Error).throw_into(env)`
#[derive(Debug, Clone)]
pub struct Error<S: AsRef<str> = Status> {
  pub status: S,
  pub reason: String,
  // Convert raw `JsError` into Error
  pub(crate) maybe_raw: libnode_sys::napi_ref,
}

impl<S: AsRef<str>> ToNapiValue for Error<S> {
  unsafe fn to_napi_value(
    env: libnode_sys::napi_env,
    val: Self,
  ) -> Result<libnode_sys::napi_value> {
    if val.maybe_raw.is_null() {
      let err = unsafe { JsError::from(val).into_value(env) };
      Ok(err)
    } else {
      let mut value = std::ptr::null_mut();
      check_status!(
        unsafe { libnode_sys::napi_get_reference_value(env, val.maybe_raw, &mut value) },
        "Get error reference in `to_napi_value` failed"
      )?;
      check_status!(
        unsafe { libnode_sys::napi_delete_reference(env, val.maybe_raw) },
        "Delete error reference in `to_napi_value` failed"
      )?;
      Ok(value)
    }
  }
}

unsafe impl<S> Send for Error<S> where S: Send + AsRef<str> {}
unsafe impl<S> Sync for Error<S> where S: Sync + AsRef<str> {}

impl<S: AsRef<str> + std::fmt::Debug> error::Error for Error<S> {}

impl<S: AsRef<str>> From<std::convert::Infallible> for Error<S> {
  fn from(_: std::convert::Infallible) -> Self {
    unreachable!()
  }
}

#[cfg(feature = "serde-json")]
impl ser::Error for Error {
  fn custom<T: Display>(msg: T) -> Self {
    Error::new(Status::InvalidArg, msg.to_string())
  }
}

#[cfg(feature = "serde-json")]
impl de::Error for Error {
  fn custom<T: Display>(msg: T) -> Self {
    Error::new(Status::InvalidArg, msg.to_string())
  }
}

#[cfg(feature = "serde-json")]
impl From<SerdeJSONError> for Error {
  fn from(value: SerdeJSONError) -> Self {
    Error::new(Status::InvalidArg, format!("{}", value))
  }
}

impl From<JsUnknown> for Error {
  fn from(value: JsUnknown) -> Self {
    let mut result = std::ptr::null_mut();
    let status =
      unsafe { libnode_sys::napi_create_reference(value.0.env, value.0.value, 1, &mut result) };
    if status != libnode_sys::Status::napi_ok {
      return Error::new(
        Status::from(status),
        "Create Error reference failed".to_owned(),
      );
    }

    let maybe_error_message = value
      .coerce_to_string()
      .and_then(|a| a.into_utf8().and_then(|a| a.into_owned()));
    if let Ok(error_message) = maybe_error_message {
      return Self {
        status: Status::GenericFailure,
        reason: error_message,
        maybe_raw: result,
      };
    }

    Self {
      status: Status::GenericFailure,
      reason: "".to_string(),
      maybe_raw: result,
    }
  }
}

#[cfg(feature = "anyhow")]
impl From<anyhow::Error> for Error {
  fn from(value: anyhow::Error) -> Self {
    Error::new(Status::GenericFailure, format!("{:?}", value))
  }
}

impl<S: AsRef<str> + std::fmt::Debug> fmt::Display for Error<S> {
  fn fmt(
    &self,
    f: &mut fmt::Formatter<'_>,
  ) -> fmt::Result {
    if !self.reason.is_empty() {
      write!(f, "{:?}, {}", self.status, self.reason)
    } else {
      write!(f, "{:?}", self.status)
    }
  }
}

impl<S: AsRef<str>> Error<S> {
  pub fn new<R: ToString>(
    status: S,
    reason: R,
  ) -> Self {
    Error {
      status,
      reason: reason.to_string(),
      maybe_raw: ptr::null_mut(),
    }
  }

  pub fn from_status(status: S) -> Self {
    Error {
      status,
      reason: "".to_owned(),
      maybe_raw: ptr::null_mut(),
    }
  }
}

impl Error {
  pub fn from_reason<T: Into<String>>(reason: T) -> Self {
    Error {
      status: Status::GenericFailure,
      reason: reason.into(),
      maybe_raw: ptr::null_mut(),
    }
  }
}

impl From<std::ffi::NulError> for Error {
  fn from(error: std::ffi::NulError) -> Self {
    Error {
      status: Status::GenericFailure,
      reason: format!("{error}"),
      maybe_raw: ptr::null_mut(),
    }
  }
}

impl From<std::io::Error> for Error {
  fn from(error: std::io::Error) -> Self {
    Error {
      status: Status::GenericFailure,
      reason: format!("{error}"),
      maybe_raw: ptr::null_mut(),
    }
  }
}

#[derive(Clone, Debug)]
pub struct ExtendedErrorInfo {
  pub message: String,
  pub engine_reserved: *mut c_void,
  pub engine_error_code: u32,
  pub error_code: Status,
}

impl TryFrom<libnode_sys::napi_extended_error_info> for ExtendedErrorInfo {
  type Error = Error;

  fn try_from(value: libnode_sys::napi_extended_error_info) -> Result<Self> {
    Ok(Self {
      message: unsafe {
        CString::from_raw(value.error_message as *mut c_char)
          .into_string()
          .map_err(|e| Error::new(Status::GenericFailure, format!("{e}")))?
      },
      engine_error_code: value.engine_error_code,
      engine_reserved: value.engine_reserved,
      error_code: Status::from(value.error_code),
    })
  }
}

pub struct JsError<S: AsRef<str> = Status>(Error<S>);

#[cfg(feature = "anyhow")]
impl From<anyhow::Error> for JsError {
  fn from(value: anyhow::Error) -> Self {
    JsError(Error::new(Status::GenericFailure, value.to_string()))
  }
}

pub struct JsTypeError<S: AsRef<str> = Status>(Error<S>);

pub struct JsRangeError<S: AsRef<str> = Status>(Error<S>);

pub struct JsSyntaxError<S: AsRef<str> = Status>(Error<S>);

macro_rules! impl_object_methods {
  ($js_value:ident, $kind:expr) => {
    impl<S: AsRef<str>> $js_value<S> {
      /// # Safety
      ///
      /// This function is safety if env is not null ptr.
      pub unsafe fn into_value(
        self,
        env: libnode_sys::napi_env,
      ) -> libnode_sys::napi_value {
        if !self.0.maybe_raw.is_null() {
          let mut err = ptr::null_mut();
          let get_err_status =
            unsafe { libnode_sys::napi_get_reference_value(env, self.0.maybe_raw, &mut err) };
          debug_assert!(
            get_err_status == libnode_sys::Status::napi_ok,
            "Get Error from Reference failed"
          );
          let delete_err_status =
            unsafe { libnode_sys::napi_delete_reference(env, self.0.maybe_raw) };
          debug_assert!(
            delete_err_status == libnode_sys::Status::napi_ok,
            "Delete Error Reference failed"
          );
          let mut is_error = false;
          let is_error_status = unsafe { libnode_sys::napi_is_error(env, err, &mut is_error) };
          debug_assert!(
            is_error_status == libnode_sys::Status::napi_ok,
            "Check Error failed"
          );
          // make sure ref_value is a valid error at first and avoid throw error failed.
          if is_error {
            return err;
          }
        }

        let error_status = self.0.status.as_ref();
        let status_len = error_status.len();
        let reason_len = self.0.reason.len();
        let mut error_code = ptr::null_mut();
        let mut reason_string = ptr::null_mut();
        let mut js_error = ptr::null_mut();
        let create_code_status = unsafe {
          libnode_sys::napi_create_string_utf8(
            env,
            error_status.as_ptr().cast(),
            status_len as isize,
            &mut error_code,
          )
        };
        debug_assert!(create_code_status == libnode_sys::Status::napi_ok);
        let create_reason_status = unsafe {
          libnode_sys::napi_create_string_utf8(
            env,
            self.0.reason.as_ptr().cast(),
            reason_len as isize,
            &mut reason_string,
          )
        };
        debug_assert!(create_reason_status == libnode_sys::Status::napi_ok);
        let create_error_status = unsafe { $kind(env, error_code, reason_string, &mut js_error) };
        debug_assert!(create_error_status == libnode_sys::Status::napi_ok);
        js_error
      }

      pub fn into_unknown(
        self,
        env: Env,
      ) -> JsUnknown {
        let value = unsafe { self.into_value(env.raw()) };
        unsafe { JsUnknown::from_raw_unchecked(env.raw(), value) }
      }

      /// # Safety
      ///
      /// This function is safety if env is not null ptr.
      pub unsafe fn throw_into(
        self,
        env: libnode_sys::napi_env,
      ) {
        #[cfg(debug_assertions)]
        let reason = self.0.reason.clone();
        let status = self.0.status.as_ref().to_string();
        // just sure current error is pending_exception
        if status == Status::PendingException.as_ref() {
          return;
        }
        // make sure current env is not exception_pending status
        let mut is_pending_exception = false;
        assert_eq!(
          unsafe { libnode_sys::napi_is_exception_pending(env, &mut is_pending_exception) },
          libnode_sys::Status::napi_ok
        );
        let js_error = match is_pending_exception {
          true => {
            let mut error_result = std::ptr::null_mut();
            assert_eq!(
              unsafe { libnode_sys::napi_get_and_clear_last_exception(env, &mut error_result) },
              libnode_sys::Status::napi_ok
            );
            error_result
          }
          false => unsafe { self.into_value(env) },
        };
        #[cfg(debug_assertions)]
        let throw_status = unsafe { libnode_sys::napi_throw(env, js_error) };
        unsafe { libnode_sys::napi_throw(env, js_error) };
        #[cfg(debug_assertions)]
        assert!(
          throw_status == libnode_sys::Status::napi_ok,
          "Throw error failed, status: [{}], raw message: \"{}\", raw status: [{}]",
          Status::from(throw_status),
          reason,
          status
        );
      }
    }

    impl<S: AsRef<str>> From<Error<S>> for $js_value<S> {
      fn from(err: Error<S>) -> Self {
        Self(err)
      }
    }

    impl crate::napi::bindgen_prelude::ToNapiValue for $js_value {
      unsafe fn to_napi_value(
        env: libnode_sys::napi_env,
        val: Self,
      ) -> Result<libnode_sys::napi_value> {
        unsafe { ToNapiValue::to_napi_value(env, val.0) }
      }
    }
  };
}

impl_object_methods!(JsError, libnode_sys::napi_create_error);
impl_object_methods!(JsTypeError, libnode_sys::napi_create_type_error);
impl_object_methods!(JsRangeError, libnode_sys::napi_create_range_error);
impl_object_methods!(JsSyntaxError, libnode_sys::node_api_create_syntax_error);

#[doc(hidden)]
#[macro_export]
macro_rules! _error {
  ($status:expr, $($msg:tt)*) => {
    $crate::napi::Error::new($status, format!($($msg)*))
  };
}
pub use _error as error;

#[doc(hidden)]
#[macro_export]
macro_rules! _check_status {
  ($code:expr) => {{
    let c = $code;
    match c {
      libnode_sys::Status::napi_ok => Ok(()),
      _ => Err($crate::napi::Error::new($crate::napi::Status::from(c), "".to_owned())),
    }
  }};

  ($code:expr, $($msg:tt)*) => {{
    let c = $code;
    match c {
      libnode_sys::Status::napi_ok => Ok(()),
      _ => Err($crate::napi::Error::new($crate::napi::Status::from(c), format!($($msg)*))),
    }
  }};

  ($code:expr, $msg:expr, $env:expr, $val:expr) => {{
    let c = $code;
    match c {
      libnode_sys::Status::napi_ok => Ok(()),
      _ => Err($crate::napi::Error::new($crate::napi::Status::from(c), format!($msg, $crate::napi::type_of!($env, $val)?))),
    }
  }};
}
pub use _check_status as check_status;

#[doc(hidden)]
#[macro_export]
macro_rules! _check_status_and_type {
  ($code:expr, $env:ident, $val:ident, $msg:expr) => {{
    let c = $code;
    match c {
      libnode_sys::Status::napi_ok => Ok(()),
      _ => {
        use $crate::napi::js_values::NapiValue;
        let value_type = $crate::napi::type_of!($env, $val)?;
        let error_msg = match value_type {
          ValueType::Function => {
            let function_name = unsafe { JsFunction::from_raw_unchecked($env, $val).name()? };
            format!(
              $msg,
              format!(
                "function {}(..) ",
                if function_name.len() == 0 {
                  "anonymous".to_owned()
                } else {
                  function_name
                }
              )
            )
          }
          ValueType::Object => {
            let env_ = $crate::napi::Env::from($env);
            let json: $crate::napi::JSON =
              env_.get_global()?.get_named_property_unchecked("JSON")?;
            let object = json.stringify($crate::napi::JsObject($crate::napi::Value {
              value: $val,
              env: $env,
              value_type: ValueType::Object,
            }))?;
            format!($msg, format!("Object {}", object))
          }
          ValueType::Boolean | ValueType::Number => {
            let value = unsafe {
              $crate::napi::JsUnknown::from_raw_unchecked($env, $val).coerce_to_string()?
            }
            .into_utf8()?;
            format!($msg, format!("{} {} ", value_type, value.as_str()?))
          }
          ValueType::BigInt => {
            let value = unsafe {
              $crate::napi::JsUnknown::from_raw_unchecked($env, $val).coerce_to_string()?
            }
            .into_utf8()?;
            format!($msg, format!("{} {} ", value_type, value.as_str()?))
          }
          _ => format!($msg, value_type),
        };
        Err($crate::napi::Error::new(
          $crate::napi::Status::from(c),
          error_msg,
        ))
      }
    }
  }};
}
pub use _check_status_and_type as check_status_and_type;

#[doc(hidden)]
#[macro_export]
macro_rules! _check_pending_exception {
  ($env:expr, $code:expr) => {{
    use $crate::napi::NapiValue;
    let c = $code;
    match c {
      libnode_sys::Status::napi_ok => Ok(()),
      libnode_sys::Status::napi_pending_exception => {
        let mut error_result = std::ptr::null_mut();
        assert_eq!(
          unsafe { libnode_sys::napi_get_and_clear_last_exception($env, &mut error_result) },
          libnode_sys::Status::napi_ok
        );
        return Err($crate::napi::Error::from(unsafe {
          $crate::napi::bindgen_prelude::Unknown::from_raw_unchecked($env, error_result)
        }));
      }
      _ => Err($crate::napi::Error::new($crate::napi::Status::from(c), "".to_owned())),
    }
  }};

  ($env:expr, $code:expr, $($msg:tt)*) => {{
    use $crate::napi::NapiValue;
    let c = $code;
    match c {
      libnode_sys::Status::napi_ok => Ok(()),
      libnode_sys::Status::napi_pending_exception => {
        let mut error_result = std::ptr::null_mut();
        assert_eq!(
          unsafe { libnode_sys::napi_get_and_clear_last_exception($env, &mut error_result) },
          libnode_sys::Status::napi_ok
        );
        return Err($crate::napi::Error::from(unsafe {
          $crate::napi::bindgen_prelude::Unknown::from_raw_unchecked($env, error_result)
        }));
      }
      _ => Err($crate::napi::Error::new($crate::napi::Status::from(c), format!($($msg)*))),
    }
  }};
}
pub use _check_pending_exception as check_pending_exception;
