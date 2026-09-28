async function run() {
    const mixedInput = [
        null,
        undefined,
        true,
        false,
        42,
        "hello unicode 🚀",
        [1, "nested", true],
        { key: "value", num: 100 }
    ];

    const res = await parallel.map(mixedInput, "x => x");
    console.log("SERIALIZED_RESULT:" + JSON.stringify(res));

    try {
        await parallel.map([() => {}], "x => x");
        console.log("UNSUPPORTED_RESULT:UNEXPECTED_SUCCESS");
    } catch (err) {
        console.log("UNSUPPORTED_RESULT:CAUGHT_ERROR:" + err.message);
    }
}

run();
