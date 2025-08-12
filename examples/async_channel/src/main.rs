use std::time::Duration;

use edon::napi::JsNumber;
use edon::napi::JsRc;

pub fn main() -> anyhow::Result<()> {
  let nodejs = edon::Nodejs::load_auto()?;

  /*
    interface MyAsyncExtension {
      add_one(a: number): Promise<number>
    }
  */
  nodejs.napi_module_register("my_async_extension", |env, mut exports| {
    let async_fn = env.create_function_from_closure("add_one", |ctx| {
      let arg0 = ctx.get::<JsRc<JsNumber>>(0)?;
      let env = ctx.env.clone();

      // Spawn async closure that returns a Promise<JsNumber>
      ctx.env.spawn_local_promise(async move {
        // Get the value of the argument
        let value = arg0.get_int32()?;

        // Do work on separate thread and send the result back
        // via a channel
        let (tx_result, rx_result) = async_channel::unbounded::<i32>();

        // Spawn OS thread to do work on
        std::thread::spawn(move || {
          std::thread::sleep(Duration::from_secs(1));
          tx_result.send_blocking(value + 1).ok();
        });

        // Obtain result from thread and cast it to a JsNumber
        let result = rx_result.recv().await.unwrap();
        let js_value = env.create_int32(result)?;

        // Give the value back to JavaScript
        println!("Rust: Async action complete");
        Ok(js_value)
      })
    })?;

    exports.set_named_property("add_one", async_fn)?;
    Ok(exports)
  })?;

  // Start a new Nodejs context
  let ctx0 = nodejs.spawn_worker_thread()?;

  ctx0.eval(
    r#"
    const native = process._linkedBinding('my_async_extension');

    (async () => {
      const result = await native.add_one(41)
      console.log(result)
    })();
  "#,
  )?;

  Ok(())
}
