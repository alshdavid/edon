use std::ffi::CString;
use std::ptr;

use edon::sys;
use napi::{bindgen_prelude::FromNapiValue, Env, JsObject, NapiRaw};

pub fn main() -> std::io::Result<()> {
  let nodejs = edon::Nodejs::load_auto()?;

  // Register a napi module and inject it into the JavaScript runtime
  nodejs.napi_module_register("my_native_extension", |env, exports| unsafe {
    let env =  Env::from_raw(env as napi::sys::napi_env);
    let mut exports = JsObject::from_napi_value(env.raw(), exports as napi::sys::napi_value).unwrap();

    let num = env.create_int32(42).unwrap();
    exports.set_named_property("Hello", num).unwrap();

    exports.raw() as sys::napi_value
  });

  nodejs.eval(r#"
    console.log('hello world2');
    process._linkedBinding("my_native_extension");  
  "#)?;

  // let worker = nodejs.new_worker()?;
  // const code = 'console.log(process._linkedBinding("my_native_extension"))';

  // Execute JavaScript and access the native extensions via process._linkedBinding
  // worker.eval(r#"console.log('hello world')"#)?;
  // worker.eval(r#"console.log('hello world2')"#)?;

  Ok(())
}
