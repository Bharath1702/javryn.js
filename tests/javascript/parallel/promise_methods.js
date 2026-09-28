// Tests Promise .then() and .catch() chaining explicitly on parallel.map()

async function testThenCatch() {
    let thenExecuted = false;
    await parallel.map([1, 2, 3], "x => x * 10").then(res => {
        thenExecuted = true;
        console.log("THEN_RESULT:" + JSON.stringify(res));
    });

    let catchExecuted = false;
    await parallel.map([1, 2, 3], "x => { if (x === 2) throw new Error('intentional error'); return x; }").catch(err => {
        catchExecuted = true;
        console.log("CATCH_RESULT:" + err.toString());
    });

    console.log(`THEN_STATUS:${thenExecuted},CATCH_STATUS:${catchExecuted}`);
}

testThenCatch();
