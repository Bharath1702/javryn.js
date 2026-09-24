// 18_async_await.js — Async / Await functions
async function compute() {
    const a = await Promise.resolve(20);
    const b = await Promise.resolve(22);
    return a + b;
}

compute().then(result => {
    console.log("async-await-result:", result);
});
