// Compute Benchmark Example — Javryn V0.2
// Single-threaded CPU-bound Fibonacci workload to establish performance baseline.

function fibonacci(n) {
    if (n <= 1) return n;
    return fibonacci(n - 1) + fibonacci(n - 2);
}

console.log("Fibonacci(20):", fibonacci(20));
