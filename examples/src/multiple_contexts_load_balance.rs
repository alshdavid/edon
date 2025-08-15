use std::sync::mpsc::channel;

use edon::napi::JsNumber;

/*
  The purpose of this example is to demonstrate offloading Nodejs work
  across multiple Nodejs Worker thread contexts.

  This example spawns (n) Rust threads and (n) Nodejs Worker contexts
  then gets each Nodejs context to add numbers until (add_until).

  The main thread then sums the values of (add_until) and validates that
  (total) = (n)*(add_until)
*/
pub fn main() -> anyhow::Result<()> {
  // Start Nodejs
  let nodejs = edon::Nodejs::load_default(edon::auto_resolve_libnode()?)?;

  // Number of threads to spawn and number to sum until
  let threads = 5;
  let add_until = 100;

  // Rust thread handles
  let mut handles = vec![];

  for _ in 0..threads {
    // Spawn a Nodejs Worker Context
    let ctx = nodejs.spawn_worker_thread()?;

    // Spawn a Rust thread
    handles.push(std::thread::spawn(move || -> anyhow::Result<i32> {
      // Set the initial value in the JavaScript context
      ctx.eval("globalThis.sum = 0;")?;

      // Do addition in the JavaScript context using eval statements
      for _ in 0..add_until {
        ctx.eval("globalThis.sum += 1;")?;
      }

      // Extract value stored inside JavaScript using native code
      let (tx, rx) = channel();

      ctx.exec(move |env| {
        let global_this = env.get_global()?;
        let i = global_this.get_named_property::<JsNumber>("sum")?;
        tx.send(i.get_int32()?).unwrap();
        Ok(())
      })?;

      Ok(rx.recv()?)
    }));
  }

  let mut total = 0;
  for handle in handles {
    total += handle.join().unwrap()?;
  }

  println!("Summed total: {}", total);
  assert!(add_until * threads == total, "Invalid sum received");

  Ok(())
}
