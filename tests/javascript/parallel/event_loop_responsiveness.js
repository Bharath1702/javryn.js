let timerFired = false;

setTimeout(() => {
    timerFired = true;
    console.log("TIMER_FIRED:TRUE");
}, 5);

async function run() {
    const res = await parallel.map([10, 10, 10, 10], `n => {
        function fib(x) {
            if (x <= 1) return x;
            return fib(x - 1) + fib(x - 2);
        }
        return fib(n);
    }`);
    console.log("PARALLEL_DONE:" + JSON.stringify(res));
    console.log("TIMER_STATUS:" + timerFired);
}

run();
