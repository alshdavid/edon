use super::check_status;
use crate::napi::bindgen_prelude::ToNapiValue;
use crate::napi::type_of;
use crate::napi::Error;
use crate::napi::Result;

macro_rules! impl_number_conversions {
  ( $( ($name:literal, $t:ty as $st:ty, $get:ident, $create:ident) ,)* ) => {
    $(
      impl $crate::napi::bindgen_prelude::TypeName for $t {
        fn type_name() -> &'static str {
          $name
        }

        fn value_type() -> crate::napi::ValueType {
          crate::napi::ValueType::Number
        }
      }

      impl $crate::napi::bindgen_prelude::ValidateNapiValue for $t { }

      impl ToNapiValue for $t {
        unsafe fn to_napi_value(env: libnode_sys::napi_env, val: $t) -> Result<libnode_sys::napi_value> {
          let mut ptr = std::ptr::null_mut();
          let val: $st = val.into();

          check_status!(
            unsafe { libnode_sys::$create(env, val, &mut ptr) },
            "Failed to convert rust type `{}` into napi value",
            $name,
          )?;

          Ok(ptr)
        }
      }

      impl $crate::napi::bindgen_prelude::FromNapiValue for $t {
        unsafe fn from_napi_value(env: libnode_sys::napi_env, napi_val: libnode_sys::napi_value) -> Result<Self> {
          let mut ret = 0 as $st;

          check_status!(
            unsafe { libnode_sys::$get(env, napi_val, &mut ret) },
            "Failed to convert napi value {:?} into rust type `{}`",
            type_of!(env, napi_val)?,
            $name,
          )?;

          ret.try_into().map_err(|_| Error::from_reason(concat!("Failed to convert ", stringify!($st), " to ", stringify!($t))))
        }
      }
    )*
  };
}

impl_number_conversions!(
  ("u8", u8 as u32, napi_get_value_uint32, napi_create_uint32),
  ("i8", i8 as i32, napi_get_value_int32, napi_create_int32),
  ("u16", u16 as u32, napi_get_value_uint32, napi_create_uint32),
  ("i16", i16 as i32, napi_get_value_int32, napi_create_int32),
  ("u32", u32 as u32, napi_get_value_uint32, napi_create_uint32),
  ("i32", i32 as i32, napi_get_value_int32, napi_create_int32),
  ("i64", i64 as i64, napi_get_value_int64, napi_create_int64),
  ("f64", f64 as f64, napi_get_value_double, napi_create_double),
);

impl ToNapiValue for f32 {
  unsafe fn to_napi_value(
    env: libnode_sys::napi_env,
    val: f32,
  ) -> Result<libnode_sys::napi_value> {
    let mut ptr = std::ptr::null_mut();

    check_status!(
      unsafe { libnode_sys::napi_create_double(env, val.into(), &mut ptr) },
      "Failed to convert rust type `f32` into napi value",
    )?;

    Ok(ptr)
  }
}
