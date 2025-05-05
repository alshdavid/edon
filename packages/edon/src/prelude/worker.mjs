// const vm = require('node:vm');
// const nm = require('node:module');
import * as w from "node:worker_threads";
console.log(w)

// new Worker(/*javascript*/`
//   // const fs = await import('/home/dalsh/Development/alshdavid/edon/packages/edon/src/prelude/foo.mjs')
//   import '/home/dalsh/Development/alshdavid/edon/packages/edon/src/prelude/foo.mjs'
//   console.log('started')  
// `, { eval: true, type: 'commonjs' })

// const code = (`
// import '/home/dalsh/Development/alshdavid/edon/packages/edon/src/prelude/foo.mjs'
// console.log('started')    
// `)

// await import(`data:text/javascript,${encodeURIComponent(code)}`);