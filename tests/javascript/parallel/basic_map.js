const input = [1, 2, 3, 4, 5];

async function run() {
    const res = await parallel.map(input, "x => x * 10");
    console.log("RESULT:" + JSON.stringify(res));
}

run();
