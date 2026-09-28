async function run() {
    const res1 = await parallel.map([1, 2], "x => x * 2");
    const res2 = await parallel.map([3, 4], "x => x * 3");
    console.log("RESULT:" + JSON.stringify([...res1, ...res2]));
}

run();
