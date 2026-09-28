// Dedicated Backpressure Verification Script

async function testBackpressure() {
    try {
        // Submit an item array exceeding default limit (10,000)
        const hugeItems = Array.from({ length: 12000 }, (_, i) => i);
        await parallel.map(hugeItems, "x => x");
        console.log("BACKPRESSURE_RESULT:UNEXPECTED_SUCCESS");
    } catch (err) {
        console.log("BACKPRESSURE_RESULT:CAUGHT_ERROR:" + err.toString());
    }
}

testBackpressure();
