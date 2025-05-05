use std::ffi::CString;
use std::ptr;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;

use libnode_sys::napi_callback_info;
use libnode_sys::napi_env;
use libnode_sys::napi_value;

use crate::Env;

static STARTED: AtomicBool = AtomicBool::new(false);

pub enum NodejsEvent {
  EvalScript {
    code: String,
    resolve: Sender<()>,
  },
  Env {
    callback: Box<dyn FnOnce(Env) -> crate::Result<()>>,
    resolve: Sender<crate::Result<()>>,
  },
}

pub fn start_node_instance() -> crate::Result<Sender<NodejsEvent>> {
  if STARTED.compare_exchange(false, true, Ordering::Acquire, Ordering::Acquire).is_err() {
    return Err(crate::Error::NodejsAlreadyRunning)
  };

  let (tx, rx) = channel();
  let closure_data_ptr = Box::into_raw(Box::new(rx));

  /*
    interface EdonMain {
      onEvent(callback: (detail: string) => any)
    }
  */
  super::napi_module_register("edon:main", move |env, exports| unsafe {
    let name = "edon::main::onEvent";
    let mut raw_result = ptr::null_mut();
    libnode_sys::napi_create_function(
      env,
      name.as_ptr().cast(),
      name.len() as isize,
      Some(edon_prelude_main),
      closure_data_ptr.cast(),
      &mut raw_result,
    );

    libnode_sys::napi_set_named_property(
      env,
      exports.cast(),
      CString::new("onEvent").unwrap().as_ptr(),
      raw_result,
    );

    exports
  }).unwrap();

  std::thread::spawn(|| {
    super::eval_blocking(format!("{};\n", crate::prelude::MAIN_JS)).unwrap();
  });

  Ok(tx)
}

unsafe extern "C" fn edon_prelude_main(
  env: napi_env,
  info: napi_callback_info,
) -> napi_value {
  let mut n_undefined = ptr::null_mut();
  libnode_sys::napi_get_undefined(env, &mut n_undefined);

  let mut argc = 1;
  let raw_args = &mut [ptr::null_mut()];
  let mut raw_this = ptr::null_mut();
  let mut closure_data_ptr = ptr::null_mut();

  libnode_sys::napi_get_cb_info(
    env,
    info,
    &mut argc,
    raw_args.as_mut_ptr(),
    &mut raw_this,
    &mut closure_data_ptr,
  );

  let n_callback = raw_args.first().unwrap();
  let rx: &Receiver<NodejsEvent> = Box::leak(unsafe { Box::from_raw(closure_data_ptr.cast()) });

  while let Ok(event) = rx.recv() {
    match event {
      NodejsEvent::EvalScript { code, resolve } => {
        let mut code_value = ptr::null_mut();
        libnode_sys::napi_create_string_utf8(
          env,
          code.as_ptr().cast(),
          code.len() as isize,
          &mut code_value,
        );

        libnode_sys::napi_call_function(
          env,
          n_undefined,
          n_callback.cast(),
          1,
          [code_value].as_mut_ptr(),
          ptr::null_mut(),
        );

        resolve.send(()).unwrap();
      }
      NodejsEvent::Env { callback, resolve } => {
        let env = Env::from_raw(env);
        resolve.send(callback(env)).ok();
      },
    }
  }

  n_undefined
}
