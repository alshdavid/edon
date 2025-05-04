pub use crate::async_work::AsyncWorkPromise;
pub use crate::bindgen_runtime::iterator;
pub use crate::call_context::CallContext;
pub use crate::env::*;
pub use crate::error::*;
pub use crate::js_values::*;
pub use crate::status::Status;
pub use crate::task::Task;
pub use crate::value_type::*;
pub use crate::version::NodeVersion;
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
    check_status!(unsafe { $crate::sys::napi_typeof($env, $value, &mut value_type) })
      .and_then(|_| Ok($crate::ValueType::from(value_type)))
  }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! assert_type_of {
  ($env: expr, $value:expr, $value_ty: expr) => {
    $crate::type_of!($env, $value).and_then(|received_type| {
      if received_type == $value_ty {
        Ok(())
      } else {
        Err($crate::Error::new(
          $crate::Status::InvalidArg,
          format!(
            "Expect value to be {}, but received {}",
            $value_ty, received_type
          ),
        ))
      }
    })
  };
}

pub use crate::bindgen_runtime::ctor as module_init;

pub mod bindgen_prelude {
  pub use crate::assert_type_of;
  pub use crate::bindgen_runtime::*;
  pub use crate::check_pending_exception;
  pub use crate::check_status;
  pub use crate::check_status_or_throw;
  pub use crate::error;
  pub use crate::error::*;
  pub use crate::sys;
  pub use crate::type_of;
  pub use crate::JsError;
  pub use crate::Property;
  pub use crate::PropertyAttributes;
  pub use crate::Result;
  pub use crate::Status;
  pub use crate::Task;
  pub use crate::ValueType;
}

#[doc(hidden)]
pub mod __private {
  pub use crate::bindgen_runtime::get_class_constructor;
  pub use crate::bindgen_runtime::iterator::create_iterator;
  pub use crate::bindgen_runtime::register_class;
  pub use crate::bindgen_runtime::___CALL_FROM_FACTORY;
  use crate::sys;

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
