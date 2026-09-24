// 07_loops.js — Control Loops
let total = 0;
for (let i = 1; i <= 5; i++) {
    total += i;
}
console.log(total);

let items = ["a", "b", "c"];
let out = [];
for (const item of items) {
    out.push(item.toUpperCase());
}
console.log(out.join("-"));
