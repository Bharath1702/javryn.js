// 26_worker_message.js — Worker postMessage communication
const worker = new Worker("./tests/javascript/workers/message.js");
worker.onmessage = (e) => {
    console.log("received:", e.data);
    worker.terminate();
};
worker.postMessage(32);
