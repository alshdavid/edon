use std::ffi::CString;
use std::ptr;
use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;

use crate::sys::napi::napi_callback_info;
use crate::sys::napi::napi_env;
use crate::sys::napi::napi_value;
use crate::sys::{self};

pub fn start_node_instance() -> Sender<(String, Sender<()>)> {
  let (tx, rx) = channel();
  let closure_data_ptr = Box::into_raw(Box::new(rx));

  super::napi_module_register("edon:main", move |env, exports| unsafe {
    let name = "edon::main::onEvent";
    let mut raw_result = ptr::null_mut();
    sys::napi::napi_create_function(
      env,
      name.as_ptr().cast(),
      name.len() as isize,
      Some(edon_prelude_main),
      closure_data_ptr.cast(),
      &mut raw_result,
    );

    // Set number on exports object
    sys::napi::napi_set_named_property(
      env,
      exports.cast(),
      CString::new("onEvent").unwrap().as_ptr(),
      raw_result,
    );

    // Return exports object
    exports
  });

  std::thread::spawn(|| {
    super::eval_blocking(format!("{};\n", super::super::prelude::MAIN_JS)).unwrap();
  });

  // thread::sleep(Duration::from_secs(1));

  tx
}

unsafe extern "C" fn edon_prelude_main(
  env: napi_env,
  info: napi_callback_info,
) -> napi_value {
  let mut n_undefined = ptr::null_mut();
  sys::napi::napi_get_undefined(env, &mut n_undefined);

  let mut argc = 1;
  let raw_args = &mut [ptr::null_mut()];
  let mut raw_this = ptr::null_mut();
  let mut closure_data_ptr = ptr::null_mut();

  sys::napi::napi_get_cb_info(
    env,
    info,
    &mut argc,
    raw_args.as_mut_ptr(),
    &mut raw_this,
    &mut closure_data_ptr,
  );

  let n_callback = raw_args.get(0).unwrap();
  let rx: &Receiver<(String, Sender<()>)> =
    Box::leak(unsafe { Box::from_raw(closure_data_ptr.cast()) });

  while let Ok((code, tx_resolved)) = rx.recv() {
    let mut code_value = ptr::null_mut();
    sys::napi::napi_create_string_utf8(
      env,
      code.as_ptr().cast(),
      code.len() as isize,
      &mut code_value,
    );

    sys::napi::napi_call_function(
      env,
      n_undefined,
      n_callback.cast(),
      1,
      [code_value].as_mut_ptr(),
      ptr::null_mut(),
    );

    tx_resolved.send(()).unwrap();
  }

  n_undefined
}
