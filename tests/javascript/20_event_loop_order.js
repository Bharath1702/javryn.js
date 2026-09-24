// 20_event_loop_order.js — Event Loop Ordering Test
// Order: Script turn -> Microtasks -> Timers -> Microtasks enqueued by timer
console.log("A");

Promise.resolve().then(() => {
    console.log("B");
});

setTimeout(() => {
    console.log("C");
    Promise.resolve().then(() => {
        console.log("D");
    });
}, 10);

console.log("E");
