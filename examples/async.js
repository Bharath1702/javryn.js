// Async Event Loop Example — Javryn V0.3
console.log("start");

Promise.resolve().then(() => {
    console.log("microtask");
});

setTimeout(() => {
    console.log("timer");
}, 50);

console.log("end");
