void (function () {
  const { Worker, workerData } = require("node:worker_threads");

  const cjsWorker = /*javascript*/`
    const vm = require("node:vm");
    const module = require("node:module");
    const { parentPort, workerData } = require("node:worker_threads");

    // Shim for "_linkedBinding"
    globalThis.importNative = (specifier) => process._linkedBinding(specifier);
    
    // HACK: If the thread exits before the host process
    // has time to clean up then Nodejs will throw a segfault
    setInterval(() => {}, 1000)

    process
      ._linkedBinding("edon:worker")
      .onEvent(workerData, async (action, payload, done) => {
          switch (action) {
            case 0: {
              const script = new vm.Script(module.stripTypeScriptTypes(payload), {
                filename: "edon::eval::script.vm",
              });
              done(await script.runInThisContext({
                breakOnSigint: true,
                displayErrors: true,
              }))
              break;
            }
            case 1: {
              require(payload)
              done()
              break;
            }
            case 2: {
              await import(payload)
              done()
              break;
            }
            default:
              break;
          }
      });

    parentPort.postMessage(null)
  `

  const wrks = []

  // Handle requests from the host
  process
    ._linkedBinding("edon:main")
    .onEvent((action, payload, done) => {
      switch (action) {
        case 0: {
          let wrk = new Worker(cjsWorker, { workerData: payload, eval: true })
          wrk.once('message', done)
          wrk.ref()
          wrks.push(wrks)
          break;
        }
        default:
          break;
      }
    });

})();
