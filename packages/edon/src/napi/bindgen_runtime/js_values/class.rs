use std::any::type_name;
use std::ops::Deref;
use std::ops::DerefMut;
use std::ptr;

use libnode_sys;

use super::Object;
use crate::napi::bindgen_runtime::FromNapiValue;
use crate::napi::bindgen_runtime::TypeName;
use crate::napi::bindgen_runtime::ValidateNapiValue;
use crate::napi::check_status;
use crate::napi::Env;
use crate::napi::NapiRaw;
use crate::napi::NapiValue;
use crate::napi::ValueType;

pub type This<T = Object> = T;

pub struct ClassInstance<T: 'static> {
  pub value: libnode_sys::napi_value,
  inner: &'static mut T,
}

impl<T: 'static> ClassInstance<T> {
  #[doc(hidden)]
  pub fn new(
    value: libnode_sys::napi_value,
    inner: &'static mut T,
  ) -> Self {
    Self { value, inner }
  }

  pub fn as_object(
    &self,
    env: Env,
  ) -> Object {
    unsafe { Object::from_raw_unchecked(env.raw(), self.value) }
  }
}

impl<T: 'static> NapiRaw for ClassInstance<T> {
  unsafe fn raw(&self) -> libnode_sys::napi_value {
    self.value
  }
}

impl<T: 'static> TypeName for ClassInstance<T>
where
  &'static T: TypeName,
{
  fn type_name() -> &'static str {
    type_name::<&T>()
  }

  fn value_type() -> ValueType {
    <&T>::value_type()
  }
}

impl<T: 'static> ValidateNapiValue for ClassInstance<T>
where
  &'static T: ValidateNapiValue,
{
  unsafe fn validate(
    env: libnode_sys::napi_env,
    napi_val: libnode_sys::napi_value,
  ) -> crate::napi::Result<libnode_sys::napi_value> {
    unsafe { <&T>::validate(env, napi_val) }
  }
}

impl<T: 'static> FromNapiValue for ClassInstance<T> {
  unsafe fn from_napi_value(
    env: libnode_sys::napi_env,
    napi_val: libnode_sys::napi_value,
  ) -> crate::napi::Result<Self> {
    let mut value = ptr::null_mut();
    check_status!(
      unsafe { libnode_sys::napi_unwrap(env, napi_val, &mut value) },
      "Unwrap value [{}] from class failed",
      type_name::<T>(),
    )?;
    let value = unsafe { Box::from_raw(value as *mut T) };
    Ok(Self {
      value: napi_val,
      inner: Box::leak(value),
    })
  }
}

impl<T: 'static> Deref for ClassInstance<T> {
  type Target = T;

  fn deref(&self) -> &Self::Target {
    self.inner
  }
}

impl<T: 'static> DerefMut for ClassInstance<T> {
  fn deref_mut(&mut self) -> &mut T {
    self.inner
  }
}

impl<T: 'static> AsRef<T> for ClassInstance<T> {
  fn as_ref(&self) -> &T {
    self.inner
  }
}

impl<T: 'static> AsMut<T> for ClassInstance<T> {
  fn as_mut(&mut self) -> &mut T {
    self.inner
  }
}
