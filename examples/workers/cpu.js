// examples/workers/cpu.js — Independent CPU compute in worker
function fibonacci(n) {
    if (n <= 1) return n;
    return fibonacci(n - 1) + fibonacci(n - 2);
}

onmessage = (event) => {
    const res = fibonacci(event.data);
    postMessage(res);
};
