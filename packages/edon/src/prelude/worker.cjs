const vm = require('node:vm');
const nm = require('node:module');

(async () => {

globalThis.foo = 'bar'

const mod = new vm.SourceTextModule(`
  console.log(globalThis.foo)
`, {
  importModuleDynamically: (...args) => {
    return import(args[0])
  }
});

const imports = new Map();
async function linker(specifier, referencingModule) {
  if (imports.has(specifier))
    return imports.get(specifier);
  
  const mod = await import(specifier);
  const exportNames = Object.keys(mod);
  const imported = new vm.SyntheticModule(
    exportNames,
    function () {
      exportNames.forEach(key => imported.setExport(key, mod[key]));
    },
    { identifier: specifier, context: referencingModule.context }
  );

  imports.set(specifier, imported);
  return imported;
}

await mod.link(linker);
console.log(await mod.evaluate());
console.log(mod)

})()
