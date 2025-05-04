use super::Value;
use crate::napi::bindgen_runtime::TypeName;
use crate::napi::bindgen_runtime::ValidateNapiValue;
use crate::napi::ValueType;

#[derive(Clone, Copy)]
pub struct JsUndefined(pub(crate) Value);

impl TypeName for JsUndefined {
  fn type_name() -> &'static str {
    "undefined"
  }

  fn value_type() -> crate::napi::ValueType {
    ValueType::Undefined
  }
}

impl ValidateNapiValue for JsUndefined {}
