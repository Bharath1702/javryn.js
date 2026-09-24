// 02_functions.js — Functions & Arrow Functions
function add(x, y) {
    return x + y;
}

const multiply = (x, y = 2) => x * y;

console.log(add(5, 5));
console.log(multiply(4));
console.log(multiply(4, 3));
