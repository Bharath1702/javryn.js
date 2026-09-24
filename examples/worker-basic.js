// examples/worker-basic.js — Basic Worker creation manual test
console.log("main runtime");
const worker = new Worker("./examples/workers/basic.js");
