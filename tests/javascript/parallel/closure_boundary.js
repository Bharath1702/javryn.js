async function run() {
    try {
        await parallel.map([1, 2, 3], "x => x + outerVar");
        console.log("CLOSURE_RESULT:UNEXPECTED_SUCCESS");
    } catch (err) {
        console.log("CLOSURE_RESULT:CAUGHT_ERROR:" + err.message);
    }
}

run();
