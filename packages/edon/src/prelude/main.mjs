void (function () {
  // This is a shim that adds in the functionality 
  // which will be added into libnode later
  const { Worker } = require("node:worker_threads");

  const cjsWorker = /*javascript*/`
    const vm = require("node:vm");
    const module = require("node:module");
    const { parentPort, workerData } = require("node:worker_threads");

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

  const workers = []

  // Handle requests from the host
  process
    ._linkedBinding("edon:main")
    .onEvent((action, payload, done) => {
      switch (action) {
        case 0: {
          let worker = new Worker(cjsWorker, { workerData: payload, eval: true })
          worker.ref()
          workers.push(worker)
          worker.once('message', done)
          break;
        }
        default:
          break;
      }
    });

})();
