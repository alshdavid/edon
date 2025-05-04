use super::ValueType;
use crate::sys;

#[derive(Clone, Copy)]
pub struct Value {
  pub env: sys::napi_env,
  pub value: sys::napi_value,
  pub value_type: ValueType,
}
