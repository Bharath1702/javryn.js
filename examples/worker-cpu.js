// examples/worker-cpu.js — Multiple independent CPU workers test
const w1 = new Worker("./examples/workers/cpu.js");
const w2 = new Worker("./examples/workers/cpu.js");

let doneCount = 0;
function checkDone() {
    doneCount++;
    if (doneCount === 2) {
        w1.terminate();
        w2.terminate();
    }
}

w1.onmessage = (event) => {
    console.log("worker 1 fib(20):", event.data);
    checkDone();
};

w2.onmessage = (event) => {
    console.log("worker 2 fib(20):", event.data);
    checkDone();
};

w1.postMessage(20);
w2.postMessage(20);
