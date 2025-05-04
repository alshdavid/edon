use std::convert::TryFrom;

use super::Value;
use crate::napi::bindgen_runtime::TypeName;
use crate::napi::bindgen_runtime::ValidateNapiValue;
use crate::napi::check_status;
use libnode_sys;
use crate::napi::Error;
use crate::napi::Result;
use crate::napi::ValueType;

#[derive(Clone, Copy)]
pub struct JsBoolean(pub(crate) Value);

impl TypeName for JsBoolean {
  fn type_name() -> &'static str {
    "bool"
  }

  fn value_type() -> crate::napi::ValueType {
    ValueType::Boolean
  }
}

impl ValidateNapiValue for JsBoolean {}

impl JsBoolean {
  pub fn get_value(&self) -> Result<bool> {
    let mut result = false;
    check_status!(unsafe { libnode_sys::napi_get_value_bool(self.0.env, self.0.value, &mut result) })?;
    Ok(result)
  }
}

impl TryFrom<JsBoolean> for bool {
  type Error = Error;

  fn try_from(value: JsBoolean) -> Result<bool> {
    value.get_value()
  }
}
