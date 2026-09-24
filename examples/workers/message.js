// examples/workers/message.js — Child worker handling onmessage and postMessage
onmessage = (event) => {
    console.log("worker received:", event.data);
    postMessage(event.data * 2);
};
