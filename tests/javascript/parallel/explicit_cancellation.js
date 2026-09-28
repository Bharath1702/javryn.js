// Dedicated Explicit Operation Cancellation Verification Script

async function testExplicitCancellation() {
    try {
        // Submit an operation with slow items
        const items = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        const p = parallel.map(items, "x => { let s = 0; for(let i=0; i<100000; i++) s += i; return x * 2; }");

        // Explicitly cancel Operation ID 1
        const cancelResult = parallel.cancel(1);
        console.log("EXPLICIT_CANCEL_TRIGGERED:" + cancelResult);

        await p;
        console.log("EXPLICIT_CANCEL_RESULT:UNEXPECTED_SUCCESS");
    } catch (err) {
        console.log("EXPLICIT_CANCEL_RESULT:CAUGHT_ERROR:" + err.toString());
    }

    // Submit a new operation to prove system recovery after explicit cancellation
    const res = await parallel.map([10, 20], "x => x * 2");
    console.log("POST_CANCEL_RECOVERY:" + JSON.stringify(res));
}

testExplicitCancellation();
