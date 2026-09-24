// 24_timer_errors.js — Exception inside timer callback
try {
    throw new Error("sync error inside catch");
} catch (e) {
    console.log("handled-timer-error-setup");
}
