// 06_closures.js — Closures
function makeCounter(start) {
    let count = start;
    return function() {
        count += 1;
        return count;
    };
}

const counter = makeCounter(10);
console.log(counter());
console.log(counter());
console.log(counter());
