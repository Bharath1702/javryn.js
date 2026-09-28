// V0.7 Scheduler fairness, priority, and uneven workload test

async function testSchedulerFairness() {
    // Test 1: Uneven workload tasks
    const items = [10, 1, 15, 2, 20, 3, 25, 4];
    const unevenResults = await parallel.map(items, "x => { let sum = 0; for (let i = 0; i < x * 1000; i++) { sum += i; } return x * 2; }");
    console.log("UNEVEN_WORKLOAD_RESULT:" + JSON.stringify(unevenResults));

    // Test 2: Concurrent operations with multi-task load
    const op1 = parallel.map([1, 2, 3, 4], "x => x + 10");
    const op2 = parallel.map([100, 200, 300], "x => x * 2");
    const [res1, res2] = await Promise.all([op1, op2]);

    console.log("SCHEDULER_CONCURRENT_OP1:" + JSON.stringify(res1));
    console.log("SCHEDULER_CONCURRENT_OP2:" + JSON.stringify(res2));
}

testSchedulerFairness();
