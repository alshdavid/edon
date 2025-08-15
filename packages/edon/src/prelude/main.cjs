void (function () {
  // This is a shim that adds in the functionality 
  // which will possibly be added into libnode later
  const { Worker } = require("node:worker_threads");

  const cjsWorker = /*javascript*/`
    const module = require("node:module");
    const process = require("node:process");
    const { parentPort, workerData } = require("node:worker_threads");

    let active = true;

    process
      ._linkedBinding("edon:worker")
      .onEvent(workerData, async (action, payload, done) => {
        if (!active) {
          // TODO return error
          // This shouldn't happen though it's nice just in case
          throw new Error("Context shutting down");
        }
        // Do each action asynchronously
        setTimeout(async () => {
          switch (action) {
            // NodejsWorkerEvent::Eval
            case 0: {
              done(eval(payload));
              break;
            }
            // NodejsWorkerEvent::EvalTypeScript
            case 1: {
              done(eval(module.stripTypeScriptTypes(payload)));
              break;
            }
            // NodejsWorkerEvent::Require
            case 2: {
              require(payload);
              done();
              break;
            }
            // NodejsWorkerEvent::Import
            case 3: {
              await import(payload);
              done();
              break;
            }
          }
        }, 0);
      });

    parentPort.once("message", async () => {
      active = false;
      process.stdout.end();
      process.stderr.end();
      parentPort.postMessage(null);
    });

    parentPort.postMessage(null);
  `

  const workers = {}

  // Handle requests from the host
  process
    ._linkedBinding("edon:main")
    .onEvent(async (action, payload, done) => {
      switch (action) {
        // NodejsMainEvent::StopMain
        case 0: {
          for (const worker of Object.values(workers)) {
            const onend = new Promise(res => worker.once('message', res))
            worker.postMessage(null)
            await onend
            const onclose = new Promise(res => worker.once('exit', res))
            await worker.terminate()
            await onclose
          }
          // Flush promises
          await new Promise(res => setTimeout(res, 0))
          done()
          break
        }  
        // NodejsMainEvent::Eval
        case 1: {
          done(eval(payload));
          break;
        }
        // NodejsMainEvent::EvalTypeScript
        case 2: {
          done(eval(module.stripTypeScriptTypes(payload)));
          break;
        }
        // NodejsMainEvent::Require
        case 3: {
          require(payload);
          done();
          break;
        }
        // NodejsMainEvent::Import
        case 4: {
          await import(payload);
          done();
          break;
        }
        // NodejsMainEvent::StartWorker
        case 5: {
          const [argv, tx_worker] = payload

          let worker = new Worker(cjsWorker, {
            argv,
            workerData: tx_worker,
            eval: true,
            stderr: true,
            stdout: true,
            stdin: false,
          })

          worker.ref()
          workers[worker.threadId] = worker
          worker.stdout.on('data', d => process.stdout.write(d))
          worker.stderr.on('data', d => process.stderr.write(d))

          await new Promise(res => worker.once('message', res))
          done(`${worker.threadId}`)
          break
        }
        // NodejsMainEvent::StopWorker
        case 6: {
          if (workers[payload]) {
            const onend = new Promise(res => workers[payload].once('message', res))
            workers[payload].postMessage(null)
            await onend
            await workers[payload].terminate()
            delete workers[payload]
          }
          done()
          break
        }        
      }
    });
})();
