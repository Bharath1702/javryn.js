// 22_interval.js — setInterval and clearInterval test
let count = 0;

const intervalId = setInterval(() => {
    count += 1;
    console.log("interval-tick:", count);
    if (count === 3) {
        clearInterval(intervalId);
    }
}, 10);
