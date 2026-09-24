// 19_microtasks.js — queueMicrotask API
console.log("main-script");

queueMicrotask(() => {
    console.log("microtask-1");
});

queueMicrotask(() => {
    console.log("microtask-2");
});

console.log("main-script-end");
