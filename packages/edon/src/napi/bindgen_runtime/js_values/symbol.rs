use std::ptr;

use libnode_sys;

use super::FromNapiValue;
use super::ToNapiValue;
use super::TypeName;
use super::ValidateNapiValue;
use crate::napi::check_status;

pub struct Symbol {
  desc: Option<String>,
  for_desc: Option<String>,
}

impl TypeName for Symbol {
  fn type_name() -> &'static str {
    "Symbol"
  }

  fn value_type() -> crate::napi::ValueType {
    crate::napi::ValueType::Symbol
  }
}

impl ValidateNapiValue for Symbol {}

impl Symbol {
  pub fn new(desc: String) -> Self {
    Self {
      desc: Some(desc),
      for_desc: None,
    }
  }

  pub fn identity() -> Self {
    Self {
      desc: None,
      for_desc: None,
    }
  }

  pub fn for_desc(desc: String) -> Self {
    Self {
      desc: None,
      for_desc: Some(desc.to_owned()),
    }
  }
}

impl ToNapiValue for Symbol {
  unsafe fn to_napi_value(
    env: libnode_sys::napi_env,
    val: Self,
  ) -> crate::napi::Result<libnode_sys::napi_value> {
    let mut symbol_value = ptr::null_mut();
    if let Some(desc) = val.for_desc {
      check_status!(
        unsafe {
          libnode_sys::node_api_symbol_for(
            env,
            desc.as_ptr().cast(),
            desc.len() as isize,
            &mut symbol_value,
          )
        },
        "Failed to call node_api_symbol_for"
      )?;
      return Ok(symbol_value);
    }
    check_status!(unsafe {
      libnode_sys::napi_create_symbol(
        env,
        match val.desc {
          Some(desc) => {
            let mut desc_string = ptr::null_mut();
            let desc_len = desc.len();
            check_status!(libnode_sys::napi_create_string_utf8(
              env,
              desc.as_ptr().cast(),
              desc_len as isize,
              &mut desc_string
            ))?;
            desc_string
          }
          None => ptr::null_mut(),
        },
        &mut symbol_value,
      )
    })?;
    Ok(symbol_value)
  }
}

impl FromNapiValue for Symbol {
  unsafe fn from_napi_value(
    _env: libnode_sys::napi_env,
    _napi_val: libnode_sys::napi_value,
  ) -> crate::napi::Result<Self> {
    Ok(Self {
      desc: None,
      for_desc: None,
    })
  }
}
