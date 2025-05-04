use crate::napi::bindgen_prelude::*;
use crate::napi::check_status;
use libnode_sys;
use crate::napi::ValueType;

impl TypeName for bool {
  fn type_name() -> &'static str {
    "bool"
  }

  fn value_type() -> ValueType {
    ValueType::Boolean
  }
}

impl ValidateNapiValue for bool {}

impl ToNapiValue for bool {
  unsafe fn to_napi_value(
    env: libnode_sys::napi_env,
    val: bool,
  ) -> Result<libnode_sys::napi_value> {
    let mut ptr = std::ptr::null_mut();

    check_status!(
      unsafe { libnode_sys::napi_get_boolean(env, val, &mut ptr) },
      "Failed to convert rust type `bool` into napi value",
    )?;

    Ok(ptr)
  }
}

impl FromNapiValue for bool {
  unsafe fn from_napi_value(
    env: libnode_sys::napi_env,
    napi_val: libnode_sys::napi_value,
  ) -> Result<Self> {
    let mut ret = false;

    check_status!(
      unsafe { libnode_sys::napi_get_value_bool(env, napi_val, &mut ret) },
      "Failed to convert napi value into rust type `bool`",
    )?;

    Ok(ret)
  }
}
