// 17_promises.js — Promise resolution chain
console.log("promise-start");

Promise.resolve(100)
    .then(val => val * 2)
    .then(val => {
        console.log("promise-chain:", val);
    });

console.log("promise-end");
