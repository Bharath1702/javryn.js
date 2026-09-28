//! Automatic parallelization module for Javryn V0.8.

pub mod analyzer;
pub mod chunker;

pub use analyzer::{ParallelizationDecision, StaticAnalyzer};
pub use chunker::calculate_chunk_size;

use std::sync::atomic::{AtomicBool, Ordering};

static AUTO_PARALLEL_ENABLED: AtomicBool = AtomicBool::new(false);

/// Sets whether automatic parallelization mode is globally enabled for the runtime session.
pub fn set_auto_parallel_enabled(enabled: bool) {
    AUTO_PARALLEL_ENABLED.store(enabled, Ordering::Relaxed);
}

/// Returns `true` if automatic parallelization mode is active.
pub fn is_auto_parallel_enabled() -> bool {
    AUTO_PARALLEL_ENABLED.load(Ordering::Relaxed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auto_parallel_toggle() {
        set_auto_parallel_enabled(true);
        assert!(is_auto_parallel_enabled());
        set_auto_parallel_enabled(false);
        assert!(!is_auto_parallel_enabled());
    }

    #[test]
    fn test_analyzer_pure_expression() {
        let analyzer = StaticAnalyzer::new();
        let decision = analyzer.analyze_source("x => x * 2");
        assert!(matches!(decision, ParallelizationDecision::Parallel { .. }));
    }

    #[test]
    fn test_analyzer_side_effect_rejection() {
        let analyzer = StaticAnalyzer::new();
        let decision = analyzer.analyze_source("x => { console.log(x); return x * 2; }");
        assert!(matches!(
            decision,
            ParallelizationDecision::Sequential { .. }
        ));
    }

    #[test]
    fn test_analyzer_mutation_rejection() {
        let analyzer = StaticAnalyzer::new();
        let decision = analyzer.analyze_source("x => { total += x; return x; }");
        assert!(matches!(
            decision,
            ParallelizationDecision::Sequential { .. }
        ));
    }

    #[test]
    fn test_analyzer_unsupported_constructs_rejection() {
        let analyzer = StaticAnalyzer::new();

        let rejection_cases = vec![
            "x => { arr.push(x); return x; }",
            "x => { arr.pop(); return x; }",
            "x => { arr.splice(0, 1); return x; }",
            "x => { globalThis.counter++; return x; }",
            "x => { window.location = 'test'; return x; }",
            "x => { fetch('http://api'); return x; }",
            "x => { setTimeout(() => {}, 10); return x; }",
            "x => { async function f() {} return x; }",
            "x => { await f(); return x; }",
            "x => { try { f(); } catch(e) {} return x; }",
            "x => { throw new Error(); }",
            "x => { break; }",
            "x => { continue; }",
            "x => { output[0] = x; }",
        ];

        for case in rejection_cases {
            let decision = analyzer.analyze_source(case);
            assert!(
                matches!(decision, ParallelizationDecision::Sequential { .. }),
                "Expected sequential fallback for: {case}"
            );
        }
    }
}
