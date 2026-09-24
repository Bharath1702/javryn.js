// Async / Await Example — Javryn V0.3
async function main() {
    console.log("before");

    const value = await Promise.resolve(42);

    console.log("after", value);
}

main();
