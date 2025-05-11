void (function () {
  // This is a shim that adds in the functionality 
  // which will be added into libnode later
  const { Worker } = require("node:worker_threads");

  const cjsWorker = /*javascript*/`
    const vm = require("node:vm");
    const module = require("node:module");
    const process = require("node:process");
    const { parentPort, workerData } = require("node:worker_threads");

    const importsCache = new Map();

    async function importsLinker(specifier, referencingModule) {
      if (importsCache.has(specifier))
        return importsCache.get(specifier);
      
      const mod = await import(specifier);
      const exportNames = Object.keys(mod);
      const imported = new vm.SyntheticModule(
        exportNames,
        function () {
          exportNames.forEach(key => imported.setExport(key, mod[key]));
        },
        { identifier: specifier, context: referencingModule.context }
      );
    
      importsCache.set(specifier, imported);
      return imported;
    }

    async function importsDynLinker(specifier, referencingModule) {
      const m = await importsLinker(specifier, referencingModule)
      if (m.status === 'unlinked') {
        await m.link(importsLinker);
      }
      if (m.status === 'linked') {
        await m.evaluate();
      }
      return m;
    }

    let inProgress = new Set()
    let active = true
    
    process
      ._linkedBinding("edon:worker")
      .onEvent(workerData, async (action, payload, done_) => {
        if (!active) return
        const done = () => {
          inProgress.delete(current)
          done_()
        }
        const current = new Promise(res => setTimeout(res, 0))
          .then(async () => {
            switch (action) {
              // NodejsContextEvent::Eval
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
              // NodejsContextEvent::EvalModule
              case 1: {
                globalThis.done = done
                const module = new vm.SourceTextModule(payload, {
                  identifier: "edon::eval::module.vm", 
                  importModuleDynamically: importsDynLinker,
                })
                
                await module.link(importsLinker);
                await module.evaluate();
                done()
                break;
              }
              // NodejsContextEvent::Require
              case 2: {
                require(payload)
                done()
                break;
              }
              // NodejsContextEvent::Import
              case 3: {
                await import(payload)
                done()
                break;
              }
              default:
                break;
            }
          })
        
          inProgress.add(current)
        })

    parentPort.once('message', async () => {
      active = false
      await Promise.all(Array.from(inProgress))
      process.stdout.end()
      process.stderr.end()
      parentPort.postMessage(null)
    })

    parentPort.postMessage(null)
  `

  const workers = {}

  // Handle requests from the host
  process
    ._linkedBinding("edon:main")
    .onEvent(async (action, payload, done) => {
      // console.log({ action, payload })
      // NodejsEvent::StartCommonjsWorker
      if (action === 0) {
        let worker = new Worker(cjsWorker, { 
          workerData: payload,
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
      }

      // NodejsEvent::StopCommonjsWorker
      else if (action === 1) {
        if (workers[payload]) {
          const onend = new Promise(res => workers[payload].once('message', res))
          workers[payload].postMessage(null)
          await onend
          await workers[payload].terminate()
          delete workers[payload]
        }
        done()
      }

      // NodejsEvent::StopMain
      else if (action === 2) {
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
      }
    });
})();
