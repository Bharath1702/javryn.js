// 03_arrays.js — Array operations
const numbers = [1, 2, 3, 4, 5];
const doubled = numbers.map(x => x * 2);
const evens = doubled.filter(x => x % 4 === 0);
const sum = evens.reduce((acc, x) => acc + x, 0);

console.log(doubled.join(","));
console.log(evens.join(","));
console.log(sum);
