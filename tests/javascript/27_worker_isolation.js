// 27_worker_isolation.js — Worker global state isolation
globalThis.sharedTestVar = "main-thread-secret";
const worker = new Worker("./tests/javascript/workers/isolation.js");
worker.onmessage = (e) => {
    console.log("worker sees:", e.data); // should be "undefined"
    worker.terminate();
};
