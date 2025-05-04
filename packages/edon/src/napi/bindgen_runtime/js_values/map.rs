use std::collections::BTreeMap;
use std::collections::HashMap;
use std::hash::BuildHasher;
use std::hash::Hash;

use crate::napi::bindgen_prelude::Env;
use crate::napi::bindgen_prelude::Result;
use crate::napi::bindgen_prelude::ToNapiValue;
use crate::napi::bindgen_prelude::*;

impl<K, V, S> TypeName for HashMap<K, V, S> {
  fn type_name() -> &'static str {
    "HashMap"
  }

  fn value_type() -> ValueType {
    ValueType::Object
  }
}

impl<K: From<String> + Eq + Hash, V: FromNapiValue> ValidateNapiValue for HashMap<K, V> {}

impl<K, V, S> ToNapiValue for HashMap<K, V, S>
where
  K: AsRef<str>,
  V: ToNapiValue,
{
  unsafe fn to_napi_value(
    raw_env: libnode_sys::napi_env,
    val: Self,
  ) -> Result<libnode_sys::napi_value> {
    let env = Env::from(raw_env);
    let mut obj = env.create_object()?;
    for (k, v) in val.into_iter() {
      obj.set(k.as_ref(), v)?;
    }

    unsafe { Object::to_napi_value(raw_env, obj) }
  }
}

impl<K, V, S> FromNapiValue for HashMap<K, V, S>
where
  K: From<String> + Eq + Hash,
  V: FromNapiValue,
  S: Default + BuildHasher,
{
  unsafe fn from_napi_value(
    env: libnode_sys::napi_env,
    napi_val: libnode_sys::napi_value,
  ) -> Result<Self> {
    let obj = unsafe { Object::from_napi_value(env, napi_val)? };
    let mut map = HashMap::default();
    for key in Object::keys(&obj)?.into_iter() {
      if let Some(val) = obj.get(&key)? {
        map.insert(K::from(key), val);
      }
    }

    Ok(map)
  }
}

impl<K, V> TypeName for BTreeMap<K, V> {
  fn type_name() -> &'static str {
    "BTreeMap"
  }

  fn value_type() -> ValueType {
    ValueType::Object
  }
}

impl<K: From<String> + Ord, V: FromNapiValue> ValidateNapiValue for BTreeMap<K, V> {}

impl<K, V> ToNapiValue for BTreeMap<K, V>
where
  K: AsRef<str>,
  V: ToNapiValue,
{
  unsafe fn to_napi_value(
    raw_env: libnode_sys::napi_env,
    val: Self,
  ) -> Result<libnode_sys::napi_value> {
    let env = Env::from(raw_env);
    let mut obj = env.create_object()?;
    for (k, v) in val.into_iter() {
      obj.set(k.as_ref(), v)?;
    }

    unsafe { Object::to_napi_value(raw_env, obj) }
  }
}

impl<K, V> FromNapiValue for BTreeMap<K, V>
where
  K: From<String> + Ord,
  V: FromNapiValue,
{
  unsafe fn from_napi_value(
    env: libnode_sys::napi_env,
    napi_val: libnode_sys::napi_value,
  ) -> Result<Self> {
    let obj = unsafe { Object::from_napi_value(env, napi_val)? };
    let mut map = BTreeMap::default();
    for key in Object::keys(&obj)?.into_iter() {
      if let Some(val) = obj.get(&key)? {
        map.insert(K::from(key), val);
      }
    }

    Ok(map)
  }
}
