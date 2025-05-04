#![allow(non_upper_case_globals)]

mod async_cleanup_hook;
pub use async_cleanup_hook::AsyncCleanupHook;
mod async_work;
mod bindgen_runtime;
mod call_context;
mod cleanup_env;
mod env;
mod error;
mod js_values;
mod status;
mod task;
mod value_type;
pub mod threadsafe_function;
mod version;

pub use cleanup_env::CleanupEnvHook;

pub use crate::napi::async_work::AsyncWorkPromise;
pub use crate::napi::bindgen_runtime::iterator;
pub use crate::napi::call_context::CallContext;
pub use crate::napi::env::*;
pub use crate::napi::error::*;
pub use crate::napi::js_values::*;
pub use crate::napi::status::Status;
pub use crate::napi::task::Task;
pub use crate::napi::value_type::*;
pub use crate::napi::version::NodeVersion;

#[cfg(feature = "serde-json")]
#[macro_use]
extern crate serde;

pub type ContextlessResult<T> = Result<Option<T>>;

#[doc(hidden)]
#[macro_export(local_inner_macros)]
macro_rules! type_of {
  ($env:expr, $value:expr) => {{
    let mut value_type = 0;
    #[allow(unused_unsafe)]
    check_status!(unsafe { $libnode_sys::
napi_typeof($env, $value, &mut value_type) })
      .and_then(|_| Ok($crate::napi::ValueType::from(value_type)))
  }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! assert_type_of {
  ($env: expr, $value:expr, $value_ty: expr) => {
    $crate::napi::type_of!($env, $value).and_then(|received_type| {
      if received_type == $value_ty {
        Ok(())
      } else {
        Err($crate::napi::Error::new(
          $crate::napi::Status::InvalidArg,
          format!(
            "Expect value to be {}, but received {}",
            $value_ty, received_type
          ),
        ))
      }
    })
  };
}

pub use crate::napi::bindgen_runtime::ctor as module_init;

pub mod bindgen_prelude {
  pub use crate::napi::assert_type_of;
  pub use crate::napi::bindgen_runtime::*;
  pub use crate::napi::check_pending_exception;
  pub use crate::napi::check_status;
  pub use crate::napi::check_status_or_throw;
  pub use crate::napi::error;
  pub use crate::napi::error::*;
  pub use libnode_sys;
  pub use crate::napi::type_of;
  pub use crate::napi::JsError;
  pub use crate::napi::Property;
  pub use crate::napi::PropertyAttributes;
  pub use crate::napi::Result;
  pub use crate::napi::Status;
  pub use crate::napi::Task;
  pub use crate::napi::ValueType;
}

#[doc(hidden)]
pub mod __private {
  pub use crate::napi::bindgen_runtime::get_class_constructor;
  pub use crate::napi::bindgen_runtime::iterator::create_iterator;
  pub use crate::napi::bindgen_runtime::register_class;
  pub use crate::napi::bindgen_runtime::___CALL_FROM_FACTORY;
  use libnode_sys;

  pub unsafe fn log_js_value<V: AsRef<[sys::napi_value]>>(
    // `info`, `log`, `warning` or `error`
    method: &str,
    env: sys::napi_env,
    values: V,
  ) {
    use std::ffi::CString;
    use std::ptr;

    let mut g = ptr::null_mut();
    unsafe { sys::napi_get_global(env, &mut g) };
    let mut console = ptr::null_mut();
    let console_c_string = CString::new("console").unwrap();
    let method_c_string = CString::new(method).unwrap();
    unsafe { sys::napi_get_named_property(env, g, console_c_string.as_ptr(), &mut console) };
    let mut method_js_fn = ptr::null_mut();
    unsafe {
      sys::napi_get_named_property(env, console, method_c_string.as_ptr(), &mut method_js_fn)
    };
    unsafe {
      sys::napi_call_function(
        env,
        console,
        method_js_fn,
        values.as_ref().len(),
        values.as_ref().as_ptr(),
        ptr::null_mut(),
      )
    };
  }
}

#[cfg(feature = "error-anyhow")]
pub extern crate anyhow;
