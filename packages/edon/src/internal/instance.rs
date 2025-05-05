use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::mpsc::channel;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;


use crate::napi::threadsafe_function::ErrorStrategy;
use crate::napi::threadsafe_function::ThreadsafeFunctionCallMode;
use crate::napi::JsFunction;
use crate::napi::JsUnknown;
use crate::Env;

static STARTED: AtomicBool = AtomicBool::new(false);

pub enum NodejsEvent {
  EvalScript {
    code: String,
    resolve: Sender<()>,
  },
  EvalModule {
    code: String,
    resolve: Sender<()>,
  },
  Env {
    callback: Box<dyn Send + FnOnce(Env) -> crate::Result<()>>,
    resolve: Sender<crate::Result<()>>,
  },
}

pub fn start_node_instance() -> crate::Result<Sender<NodejsEvent>> {
  if STARTED
    .compare_exchange(false, true, Ordering::Acquire, Ordering::Acquire)
    .is_err()
  {
    return Err(crate::Error::NodejsAlreadyRunning);
  };

  let (tx, rx) = channel();
  let rx = Arc::new(Mutex::new(Some(rx)));

  super::napi_module_register("edon:main", move |env, mut exports| {
    let js_on_event = env.create_function_from_closure("edon::main::onEvent", {
      let rx = rx.clone();
      move |ctx| {
        let callback = ctx.get::<JsFunction>(0)?;

        let on_eval = callback
          .create_threadsafe_function::<NodejsEvent, JsUnknown, _, ErrorStrategy::Fatal>(
            0,
            move |ctx| {
              match ctx.value {
                NodejsEvent::EvalScript { code, resolve } => {
                  let action = ctx.env.create_uint32(0)?.into_unknown();
                  let payload = ctx.env.create_string(&code)?.into_unknown();
                  let resolve = ctx.env.create_function_from_closure("NodejsEvent::onEvent", move |ctx| {
                    resolve.send(()).unwrap();
                    ctx.env.get_undefined()
                  })?.into_unknown();
                  
                  Ok(vec![action, payload, resolve])
                }
                NodejsEvent::EvalModule { code, resolve } => {
                  let action = ctx.env.create_uint32(1)?.into_unknown();
                  let payload = ctx.env.create_string(&code)?.into_unknown();
                  let resolve = ctx.env.create_function_from_closure("NodejsEvent::onEvent", move |ctx| {
                    resolve.send(()).unwrap();
                    ctx.env.get_undefined()
                  })?.into_unknown();
                  
                  Ok(vec![action, payload, resolve])
                }
                NodejsEvent::Env { callback, resolve } => {
                  resolve.send(callback(ctx.env)).ok();
                  Ok(vec![])
                },
              }
            },
          )?;

        // on_eval.refer(&ctx.env)?;
        // on_eval.refer(&ctx.env)?;

        thread::spawn({
          let rx = rx.clone();

          move || {
            let Some(rx) = rx.lock().unwrap().take() else {
              panic!("Cannot run twice")
            };

            while let Ok(event) = rx.recv() {
              on_eval.call(event, ThreadsafeFunctionCallMode::Blocking);
            }

            println!("Loop ended")
          }
        });
        println!("called");
        Ok(())
      }
    });

    exports.set_named_property("onEvent", js_on_event)?;

    Ok(exports)
  })?;

  std::thread::spawn(move || {
    super::eval_blocking(format!("{};\n", crate::prelude::MAIN_JS)).unwrap();
  });

  Ok(tx)
}

/*
    let callback = env.create_function_from_closure("edon::main::onEval", {
      let rx = rx.clone();
      move |ctx| {
        let port2 = ctx.get::<JsObject>(0)?;
        let post_message = port2.get_named_property::<JsFunction>("postMessage")?;

        let tsfn = post_message
          .create_threadsafe_function::<NodejsEvent, _, _, ErrorStrategy::Fatal>(0, |ctx| {
            let mut obj = ctx.env.create_array_with_length(3)?;

            match ctx.value {
              NodejsEvent::EvalScript { code, resolve } => {
                let action = ctx.env.create_uint32(0)?;
                let payload = ctx.env.create_string(&code)?;
                let resolve = ctx.env.create_function_from_closure("NodejsEvent::EvalScript", move |ctx| {
                  resolve.send(()).unwrap();
                  ctx.env.get_undefined()
                })?;
                obj.set_element(0, action)?;
                obj.set_element(1, payload)?;
              }
              NodejsEvent::Env { callback, resolve } => todo!(),
            }

            Ok(vec![obj])
          })?;

        std::thread::spawn({
          let rx = rx.clone();
          move || {
            let Some(rx) = rx.lock().unwrap().take() else {
              panic!("Cannot run twice")
            };

            while let Ok(event) = rx.recv() {
              tsfn.call(event, ThreadsafeFunctionCallMode::Blocking);
            }
          }
        });
        // post_message.call::<JsUnknown>(Some(&port2), &[ctx.env.get_boolean(true)?.into_unknown()])?;

        ctx.env.get_undefined()
      }
    })?;

    // let mut tsfn = callback
    //   .create_threadsafe_function::<String,_,_,ErrorStrategy::Fatal>(0, |ctx| {
    //     println!("tsfn called");
    //     let code = ctx.env.create_string(&ctx.value)?;
    //     Ok(vec![code])
    //   })?;

    // tsfn.refer(&env)?;

    // thread::spawn(move || {
    //   while let Ok(event) = rx.recv() {
    //     match event {
    //       NodejsEvent::EvalScript { code, resolve } => {
    //         tsfn.call(code, ThreadsafeFunctionCallMode::Blocking);
    //         // let mut code_value = ptr::null_mut();
    //         resolve.send(()).unwrap();
    //       }
    //       NodejsEvent::Env { callback, resolve } => {
    //         todo!()
    //       },
    //     }
    //   }
    // });

    unsafe extern "C" fn edon_prelude_main(
  env: napi_env,
  info: napi_callback_info,
) -> napi_value {
  println!("hello word");
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
      }
    }
  }

  n_undefined
}

*/
