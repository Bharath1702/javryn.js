// 21_timer_cancellation.js — clearTimeout test
console.log("start-cancellation");

const timerId = setTimeout(() => {
    console.log("SHOULD_NOT_PRINT");
}, 10);

clearTimeout(timerId);

setTimeout(() => {
    console.log("cancellation-success");
}, 30);
