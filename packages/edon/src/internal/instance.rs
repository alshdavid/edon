use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

use crate::napi::threadsafe_function::ErrorStrategy;
use crate::napi::threadsafe_function::ThreadsafeFunctionCallMode;
use crate::napi::JsFunction;
use crate::napi::JsUnknown;
use crate::Env;

use super::JsTransferable;

static STARTED: AtomicBool = AtomicBool::new(false);

pub enum NodejsEvent {
  StartCommonjsWorker {
    rx_wrk: Receiver<NodejsContextEvent>,
    resolve: Sender<()>,
  },
}

pub enum NodejsContextEvent {
  Exec {
    callback: Box<dyn Send + FnOnce(Env) -> crate::Result<()>>,
    resolve: Sender<crate::Result<()>>,
  },
  Eval {
    code: String,
    resolve: Sender<crate::Result<()>>,
  },
  Require {
    specifier: String,
    resolve: Sender<crate::Result<()>>,
  },
  Import {
    specifier: String,
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
            move |ctx| match ctx.value {
              NodejsEvent::StartCommonjsWorker { rx_wrk, resolve } => {
                let action = ctx.env.create_uint32(0)?.into_unknown();
                let payload =
                  JsTransferable::new(Mutex::new(Some(rx_wrk))).into_unknown(&ctx.env)?;
                let resolve = ctx
                  .env
                  .create_function_from_closure("NodejsEvent::done", move |ctx| {
                    resolve.send(()).unwrap();
                    ctx.env.get_undefined()
                  })?
                  .into_unknown();

                Ok(vec![action, payload, resolve])
              }
            },
          )?;

        thread::spawn({
          let rx = rx.clone();
          move || {
            let Some(rx) = rx.lock().unwrap().take() else {
              panic!("Cannot run twice")
            };

            while let Ok(event) = rx.recv() {
              on_eval.call(event, ThreadsafeFunctionCallMode::Blocking);
            }
          }
        });

        Ok(())
      }
    });

    exports.set_named_property("onEvent", js_on_event)?;

    Ok(exports)
  })?;

  super::napi_module_register("edon:worker", move |env, mut exports| {
    let js_on_event = env.create_function_from_closure("edon::main::onEvent", |ctx| {
      let callback = ctx.get::<JsFunction>(1)?;

      let on_eval = callback
        .create_threadsafe_function::<NodejsContextEvent, JsUnknown, _, ErrorStrategy::Fatal>(
          0,
          move |ctx| match ctx.value {
            NodejsContextEvent::Exec { callback, resolve } => {
              resolve.send(callback(ctx.env)).ok();
              Ok(vec![])
            }
            NodejsContextEvent::Eval { code, resolve } => {
              let action = ctx.env.create_uint32(0)?.into_unknown();
              let payload = ctx.env.create_string(&code)?.into_unknown();
              let resolve = ctx
                .env
                .create_function_from_closure("NodejsContextEvent::done", move |ctx| {
                  resolve.send(Ok(())).unwrap();
                  ctx.env.get_undefined()
                })?
                .into_unknown();

              Ok(vec![action, payload, resolve])
            }
            NodejsContextEvent::Require { specifier, resolve } => {
              let action = ctx.env.create_uint32(1)?.into_unknown();
              let payload = ctx.env.create_string(&specifier)?.into_unknown();
              let resolve = ctx
                .env
                .create_function_from_closure("NodejsContextEvent::done", move |ctx| {
                  resolve.send(Ok(())).unwrap();
                  ctx.env.get_undefined()
                })?
                .into_unknown();

              Ok(vec![action, payload, resolve])
            },
            NodejsContextEvent::Import { specifier, resolve } => {
              let action = ctx.env.create_uint32(2)?.into_unknown();
              let payload = ctx.env.create_string(&specifier)?.into_unknown();
              let resolve = ctx
                .env
                .create_function_from_closure("NodejsContextEvent::done", move |ctx| {
                  resolve.send(Ok(())).unwrap();
                  ctx.env.get_undefined()
                })?
                .into_unknown();

              Ok(vec![action, payload, resolve])
            },
          },
        )?;

      let rx = ctx.get::<JsTransferable<Mutex<Option<Receiver<NodejsContextEvent>>>>>(0)?;
      let rx = rx.take()?.lock().unwrap().take().unwrap();

      thread::spawn({
        move || {
          while let Ok(event) = rx.recv() {
            on_eval.call(event, ThreadsafeFunctionCallMode::Blocking);
          }
        }
      });
      Ok(())
    });

    exports.set_named_property("onEvent", js_on_event)?;

    Ok(exports)
  })?;

  std::thread::spawn(move || {
    super::eval_blocking(format!("{};\n", crate::prelude::MAIN_JS)).unwrap();
  });

  Ok(tx)
}
