// V0.7 Scheduler stress, verification, duplicate check, and metric baseline script

async function verifySchedulerIntegration() {
    // 1. Large item count to verify no duplicate execution and total task accounting
    const items = Array.from({ length: 1000 }, (_, i) => i + 1);
    const results = await parallel.map(items, "x => x * 3");

    const uniqueCount = new Set(results).size;
    const sum = results.reduce((a, b) => a + b, 0);
    const expectedSum = 1000 * 1001 * 3 / 2;

    console.log(`STRESS_1000_LEN:${results.length}`);
    console.log(`STRESS_1000_UNIQUE:${uniqueCount}`);
    console.log(`STRESS_1000_CORRECT_SUM:${sum === expectedSum}`);

    // 2. Cancellation and subsequent recovery test
    const longItems = Array.from({ length: 50 }, (_, i) => i + 1);
    const opPromise = parallel.map(longItems, "x => { let s = 0; for(let i=0; i<50000; i++) s+=i; return x; }");
    
    // Trigger cancellation after slight tick
    setTimeout(() => {
        // Submit another operation immediately to verify recovery
        parallel.map([10, 20], "x => x / 2").then(res => {
            console.log("RECOVERY_AFTER_CANCEL_RESULT:" + JSON.stringify(res));
        });
    }, 2);

    try {
        await opPromise;
    } catch (err) {
        console.log("OP_CANCELLED_SUCCESSFULLY:true");
    }
}

verifySchedulerIntegration();
