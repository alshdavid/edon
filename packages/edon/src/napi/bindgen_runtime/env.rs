use std::ptr;

use super::Array;
use crate::napi::check_status;
use libnode_sys;
pub use crate::napi::Env;
use crate::napi::JsGlobal;
use crate::napi::JsNull;
use crate::napi::JsUndefined;
use crate::napi::NapiValue;
use crate::napi::Result;

impl Env {
  pub fn create_array(
    &self,
    len: u32,
  ) -> Result<Array> {
    Array::new(self.0, len)
  }

  /// Get [JsUndefined](./struct.JsUndefined.html) value
  pub fn get_undefined(&self) -> Result<JsUndefined> {
    let mut raw_value = ptr::null_mut();
    check_status!(unsafe { sys::napi_get_undefined(self.0, &mut raw_value) })?;
    let js_undefined = unsafe { JsUndefined::from_raw_unchecked(self.0, raw_value) };
    Ok(js_undefined)
  }

  pub fn get_null(&self) -> Result<JsNull> {
    let mut raw_value = ptr::null_mut();
    check_status!(unsafe { sys::napi_get_null(self.0, &mut raw_value) })?;
    let js_null = unsafe { JsNull::from_raw_unchecked(self.0, raw_value) };
    Ok(js_null)
  }

  pub fn get_global(&self) -> Result<JsGlobal> {
    let mut global = std::ptr::null_mut();
    crate::napi::check_status!(
      unsafe { sys::napi_get_global(self.0, &mut global) },
      "Get global object from Env failed"
    )?;
    Ok(JsGlobal(crate::napi::Value {
      value: global,
      env: self.0,
      value_type: crate::napi::ValueType::Object,
    }))
  }
}
