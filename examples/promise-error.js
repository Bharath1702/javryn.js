// Promise Rejection Handling Example — Javryn V0.3
Promise.resolve()
    .then(() => {
        throw new Error("async failure");
    })
    .catch(error => {
        console.log("caught:", error.message);
    });
