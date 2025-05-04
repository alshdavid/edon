use super::ValueType;
use libnode_sys;

#[derive(Clone, Copy)]
pub struct Value {
  pub env: libnode_sys::napi_env,
  pub value: libnode_sys::napi_value,
  pub value_type: ValueType,
}
