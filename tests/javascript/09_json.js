// 09_json.js — JSON Parsing and Serialization
const data = { id: 101, status: "ok", tags: ["fast", "safe"] };
const str = JSON.stringify(data);
const parsed = JSON.parse(str);

console.log(parsed.id);
console.log(parsed.status);
console.log(parsed.tags[0]);
