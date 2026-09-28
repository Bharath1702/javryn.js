const inputs = [30, 30, 30, 30];

const fibSource = `n => {
    function fib(x) {
        if (x <= 1) return x;
        return fib(x - 1) + fib(x - 2);
    }
    return fib(n);
}`;

async function main() {
    const start = Date.now();
    const results = await parallel.map(inputs, fibSource);
    const elapsed = Date.now() - start;
    console.log("CPU parallel map result:", results);
    console.log("Elapsed time (ms):", elapsed);
}

main();
