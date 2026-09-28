async function main() {
    try {
        await parallel.map([1, 2, 3, 4], `x => {
            if (x === 3) {
                throw new Error("Task failed at element 3");
            }
            return x * 2;
        }`);
        console.log("Should not reach here");
    } catch (err) {
        console.log("Caught expected parallel error:", err.message);
    }
}

main();
