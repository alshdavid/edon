use std::ptr;

use super::check_status;
use crate::napi::bindgen_runtime::TypeName;
use crate::napi::bindgen_runtime::ValidateNapiValue;
use libnode_sys;
use crate::napi::Error;
use crate::napi::Result;
use crate::napi::Status;
use crate::napi::Value;
use crate::napi::ValueType;

pub struct JsDate(pub(crate) Value);

impl TypeName for JsDate {
  fn type_name() -> &'static str {
    "Date"
  }

  fn value_type() -> crate::napi::ValueType {
    ValueType::Object
  }
}

impl ValidateNapiValue for JsDate {
  unsafe fn validate(
    env: libnode_sys::napi_env,
    napi_val: libnode_sys::napi_value,
  ) -> Result<libnode_sys::napi_value> {
    let mut is_date = false;
    check_status!(unsafe { libnode_sys::napi_is_date(env, napi_val, &mut is_date) })?;
    if !is_date {
      return Err(Error::new(
        Status::InvalidArg,
        "Expected a Date object".to_owned(),
      ));
    }

    Ok(ptr::null_mut())
  }
}

impl JsDate {
  pub fn value_of(&self) -> Result<f64> {
    let mut timestamp: f64 = 0.0;
    check_status!(unsafe { libnode_sys::napi_get_date_value(self.0.env, self.0.value, &mut timestamp) })?;
    Ok(timestamp)
  }
}
