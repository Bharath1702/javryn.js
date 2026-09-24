// Timers & Cancellation Example — Javryn V0.3
console.log("start");

const timer = setTimeout(() => {
    console.log("should-not-print");
}, 100);

clearTimeout(timer);

setTimeout(() => {
    console.log("finished");
}, 20);
