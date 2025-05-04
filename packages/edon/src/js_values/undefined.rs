use super::Value;
use crate::bindgen_runtime::TypeName;
use crate::bindgen_runtime::ValidateNapiValue;
use crate::ValueType;

#[derive(Clone, Copy)]
pub struct JsUndefined(pub(crate) Value);

impl TypeName for JsUndefined {
  fn type_name() -> &'static str {
    "undefined"
  }

  fn value_type() -> crate::ValueType {
    ValueType::Undefined
  }
}

impl ValidateNapiValue for JsUndefined {}
