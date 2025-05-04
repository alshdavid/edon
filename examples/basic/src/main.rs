use std::ffi::CString;
use std::ptr;

use edon::sys;

pub fn main() -> std::io::Result<()> {
  let nodejs = edon::Nodejs::load_auto()?;

  // Register a napi module and inject it into the JavaScript runtime
  nodejs.napi_module_register("edon:main", |env, exports| unsafe {
    // Create number
    let mut raw_value = ptr::null_mut();
    sys::napi_create_uint32(env, 42, &mut raw_value);

    // Set number on exports object
    sys::napi_set_named_property(
      env,
      exports.cast(),
      CString::new("hello").unwrap().as_ptr(),
      raw_value,
    );

    // Return exports object
    exports
  });

  let worker = nodejs.new_worker()?;
  // const code = 'console.log(process._linkedBinding("my_native_extension"))';

  // Execute JavaScript and access the native extensions via process._linkedBinding
  worker.eval(r#"console.log('hello world')"#)?;
  worker.eval(r#"console.log('hello world2')"#)?;

  Ok(())
}
