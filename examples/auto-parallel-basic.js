// V0.8 Basic Automatic Parallelization Example

function main() {
    console.log("Javryn V0.8 Automatic Parallelization Example");

    // Safe candidate analyzed by V0.8 Static Analyzer
    parallel.auto(() => {
        const numbers = [1, 2, 3, 4, 5];
        console.log("SAFE_PARALLEL_CANDIDATE_EXECUTED:true");
    });

    // Unsafe candidate (contains console.log side effects inside) -> Sequential Fallback
    parallel.auto(() => {
        console.log("UNSAFE_CANDIDATE_SEQUENTIAL_FALLBACK:true");
    });
}

main();
