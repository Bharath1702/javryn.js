async function run() {
    const p1 = parallel.map([1, 2, 3, 4], "x => x * 2");
    const p2 = parallel.map([10, 20, 30, 40], "x => x * 3");

    const [res1, res2] = await Promise.all([p1, p2]);
    console.log("CONCURRENT_1:" + JSON.stringify(res1));
    console.log("CONCURRENT_2:" + JSON.stringify(res2));
}

run();
