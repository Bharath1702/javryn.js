// 23_async_errors.js — Promise rejection handling
Promise.reject(new Error("async rejection"))
    .catch(err => {
        console.log("handled-rejection:", err.message);
    });
