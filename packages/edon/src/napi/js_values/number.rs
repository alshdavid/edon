use std::convert::TryFrom;

use libnode_sys;

use super::Value;
use crate::napi::bindgen_runtime::TypeName;
use crate::napi::bindgen_runtime::ValidateNapiValue;
use crate::napi::check_status;
use crate::napi::Error;
use crate::napi::Result;
use crate::napi::ValueType;

#[derive(Clone, Copy)]
pub struct JsNumber(pub(crate) Value);

impl TypeName for JsNumber {
  fn type_name() -> &'static str {
    "f64"
  }

  fn value_type() -> crate::napi::ValueType {
    ValueType::Number
  }
}

impl ValidateNapiValue for JsNumber {}

impl JsNumber {
  pub fn get_uint32(&self) -> Result<u32> {
    let mut result = 0;
    check_status!(unsafe {
      libnode_sys::napi_get_value_uint32(self.0.env, self.0.value, &mut result)
    })?;
    Ok(result)
  }

  pub fn get_int32(&self) -> Result<i32> {
    let mut result = 0;
    check_status!(unsafe {
      libnode_sys::napi_get_value_int32(self.0.env, self.0.value, &mut result)
    })?;
    Ok(result)
  }

  pub fn get_int64(&self) -> Result<i64> {
    let mut result = 0;
    check_status!(unsafe {
      libnode_sys::napi_get_value_int64(self.0.env, self.0.value, &mut result)
    })?;
    Ok(result)
  }

  pub fn get_double(&self) -> Result<f64> {
    let mut result = 0_f64;
    check_status!(unsafe {
      libnode_sys::napi_get_value_double(self.0.env, self.0.value, &mut result)
    })?;
    Ok(result)
  }
}

impl TryFrom<JsNumber> for u32 {
  type Error = Error;

  fn try_from(value: JsNumber) -> Result<u32> {
    let mut result = 0;
    check_status!(unsafe {
      libnode_sys::napi_get_value_uint32(value.0.env, value.0.value, &mut result)
    })?;
    Ok(result)
  }
}

impl TryFrom<JsNumber> for i32 {
  type Error = Error;

  fn try_from(value: JsNumber) -> Result<i32> {
    let mut result = 0;
    check_status!(unsafe {
      libnode_sys::napi_get_value_int32(value.0.env, value.0.value, &mut result)
    })?;
    Ok(result)
  }
}

impl TryFrom<JsNumber> for i64 {
  type Error = Error;

  fn try_from(value: JsNumber) -> Result<i64> {
    let mut result = 0;
    check_status!(unsafe {
      libnode_sys::napi_get_value_int64(value.0.env, value.0.value, &mut result)
    })?;
    Ok(result)
  }
}

impl TryFrom<JsNumber> for f64 {
  type Error = Error;

  fn try_from(value: JsNumber) -> Result<f64> {
    let mut result = 0_f64;
    check_status!(unsafe {
      libnode_sys::napi_get_value_double(value.0.env, value.0.value, &mut result)
    })?;
    Ok(result)
  }
}
