use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result;

use libnode_sys;

#[repr(i32)]
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub enum ValueType {
  Undefined = 0,
  Null = 1,
  Boolean = 2,
  Number = 3,
  String = 4,
  Symbol = 5,
  Object = 6,
  Function = 7,
  External = 8,
  BigInt = 9,
  Unknown = 1024,
}

impl Display for ValueType {
  fn fmt(
    &self,
    f: &mut Formatter<'_>,
  ) -> Result {
    let status_string = format!("{self:?}");
    write!(f, "{status_string}")
  }
}

impl From<i32> for ValueType {
  fn from(value: i32) -> ValueType {
    match value {
      libnode_sys::ValueType::napi_bigint => ValueType::BigInt,
      libnode_sys::ValueType::napi_boolean => ValueType::Boolean,
      libnode_sys::ValueType::napi_external => ValueType::External,
      libnode_sys::ValueType::napi_function => ValueType::Function,
      libnode_sys::ValueType::napi_null => ValueType::Null,
      libnode_sys::ValueType::napi_number => ValueType::Number,
      libnode_sys::ValueType::napi_object => ValueType::Object,
      libnode_sys::ValueType::napi_string => ValueType::String,
      libnode_sys::ValueType::napi_symbol => ValueType::Symbol,
      libnode_sys::ValueType::napi_undefined => ValueType::Undefined,
      _ => ValueType::Unknown,
    }
  }
}
