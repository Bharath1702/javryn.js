// 08_exceptions.js — Exception handling
try {
    throw new Error("Caught error in script");
} catch (e) {
    console.log("Handled:", e.message);
}
