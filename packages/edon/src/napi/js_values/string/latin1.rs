use std::mem::ManuallyDrop;

use crate::napi::JsString;

pub struct JsStringLatin1 {
  pub(crate) inner: JsString,
  pub(crate) buf: ManuallyDrop<Vec<u8>>,
}

impl JsStringLatin1 {
  pub fn as_slice(&self) -> &[u8] {
    &self.buf
  }

  pub fn len(&self) -> usize {
    self.buf.len()
  }

  pub fn is_empty(&self) -> bool {
    self.buf.is_empty()
  }

  pub fn take(self) -> Vec<u8> {
    self.as_slice().to_vec()
  }

  pub fn into_value(self) -> JsString {
    self.inner
  }
}

impl From<JsStringLatin1> for Vec<u8> {
  fn from(value: JsStringLatin1) -> Self {
    value.take()
  }
}
