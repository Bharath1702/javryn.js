async function run() {
    const arr = [];
    for (let i = 0; i < 1000; i++) {
        arr.push(i);
    }

    const res = await parallel.map(arr, "x => x + 1");
    console.log("LARGE_INPUT_LEN:" + res.length);
    console.log("LARGE_INPUT_FIRST:" + res[0]);
    console.log("LARGE_INPUT_LAST:" + res[999]);
}

run();
