// 28_worker_timer.js — Timer inside worker
const worker = new Worker("./tests/javascript/workers/timer.js");
worker.onmessage = (e) => {
    console.log("worker timer:", e.data);
    worker.terminate();
};
