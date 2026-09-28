async function run() {
    const res = await parallel.map([], "x => x * 2");
    console.log("RESULT:" + JSON.stringify(res));
}

run();
