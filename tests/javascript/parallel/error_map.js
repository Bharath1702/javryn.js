async function run() {
    try {
        await parallel.map([1, 2, 3], `x => {
            if (x === 2) throw new Error("bomb");
            return x;
        }`);
        console.log("RESULT:SUCCESS");
    } catch (e) {
        console.log("RESULT:ERROR:" + e.message);
    }
}

run();
