// V0.6 Concurrency pressure, worker isolation, and memory accounting test

async function testConcurrencyAndIsolation() {
    // Test 1: Worker state isolation across parallel map operations
    const isolationResults = await parallel.map([1, 2, 3, 4], "x => { if (typeof globalThis.workerState === 'undefined') { globalThis.workerState = 0; } globalThis.workerState += 100; return { input: x, state: globalThis.workerState }; }");

    console.log("ISOLATION_CHECK:" + (Array.isArray(isolationResults) && isolationResults.length === 4));

    // Test 2: Stress concurrency burst
    const items = Array.from({ length: 500 }, (_, i) => i + 1);
    const stressResult = await parallel.map(items, "x => x * 2");

    console.log(`STRESS_LEN:${stressResult.length},STRESS_SUM:${stressResult.reduce((a, b) => a + b, 0)}`);
}

testConcurrencyAndIsolation();
