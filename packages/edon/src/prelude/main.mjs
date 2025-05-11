void (function () {
  // This is a shim that adds in the functionality 
  // which will be added into libnode later
  const { Worker } = require("node:worker_threads");

  const cjsWorker = /*javascript*/`
    const vm = require("node:vm");
    const module = require("node:module");
    const { parentPort, workerData, threadId } = require("node:worker_threads");

    // HACK: If the thread exits before the host process
    // has time to clean up then Nodejs will throw a segfault
    setInterval(() => {}, 1000)

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

    process
      ._linkedBinding("edon:worker")
      .onEvent(workerData, async (action, payload, done) => setTimeout(async () => {
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
          case 2: {
            require(payload)
            done()
            break;
          }
          case 3: {
            await import(payload)
            done()
            break;
          }
          default:
            break;
        }
      }, 0));

    parentPort.postMessage(\`\${threadId}\`)
  `

  const workers = {}

  // Handle requests from the host
  process
    ._linkedBinding("edon:main")
    .onEvent(async (action, payload, done) => {
      switch (action) {
        case 0: {
          let worker = new Worker(cjsWorker, { workerData: payload, eval: true })
          worker.ref()
          worker.once('message', (id) => {
            workers[id] = worker
            done(id)
          })
          break;
        }
        default:
          if (workers[payload]) {
            await workers[payload].terminate()
            delete workers[payload]
          }
          done()
          break;
      }
    });
})();
