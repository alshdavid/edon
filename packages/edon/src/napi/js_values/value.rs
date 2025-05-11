use libnode_sys;

use super::ValueType;

#[derive(Clone, Copy)]
pub struct Value {
  pub env: libnode_sys::napi_env,
  pub value: libnode_sys::napi_value,
  pub value_type: ValueType,
}
