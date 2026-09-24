// examples/worker-message.js — Worker postMessage manual test
const worker = new Worker("./examples/workers/message.js");

worker.onmessage = (event) => {
    console.log("result:", event.data);
    worker.terminate();
};

worker.postMessage(21);
