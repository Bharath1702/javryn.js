// 04_objects.js — Objects & Destructuring
const user = { name: "Javryn", role: "Runtime", version: 0.2 };
const { name, role } = user;
const merged = { ...user, active: true };

console.log(name);
console.log(role);
console.log(Object.keys(merged).join(","));
