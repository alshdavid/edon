void (function () {
  const vm = require("node:vm");
  const module = require("node:module");
  const worker_threads = require("node:worker_threads");

  // Shim for "_linkedBinding"
  globalThis.importNative = (specifier) => process._linkedBinding(specifier);

  // Handle requests from the host
  const edon = process._linkedBinding("edon:main");
  edon.onEvent(async (action, payload, done) => {
    switch (action) {
      case 0: {
        const script = new vm.Script(module.stripTypeScriptTypes(payload), {
          filename: "edon::eval::script.vm",
        });
        done(
          await script.runInThisContext({
            breakOnSigint: true,
            displayErrors: true,
          })
        );
        break;
      }
      case 1: {
        const mod = new vm.SourceTextModule(module.stripTypeScriptTypes(payload),
          {
            importModuleDynamically: (...args) => {
              return import(args[0]);
            },
          }
        );

        const imports = new Map();
        async function linker(specifier, referencingModule) {
          if (imports.has(specifier)) return imports.get(specifier);

          const mod = await import(specifier);
          const exportNames = Object.keys(mod);
          const imported = new vm.SyntheticModule(
            exportNames,
            function () {
              exportNames.forEach((key) => imported.setExport(key, mod[key]));
            },
            { identifier: specifier, context: referencingModule.context }
          );

          imports.set(specifier, imported);
          return imported;
        }

        await mod.link(linker);
        await mod.evaluate();
        done();
        break;
      }
      default:
        break;
    }
  });
})();
