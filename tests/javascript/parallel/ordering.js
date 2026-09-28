const input = [100, 10, 50, 1];

async function run() {
    const res = await parallel.map(input, `x => {
        let dummy = 0;
        for (let i = 0; i < x * 10000; i++) {
            dummy += i;
        }
        return x;
    }`);
    console.log("RESULT:" + JSON.stringify(res));
}

run();
