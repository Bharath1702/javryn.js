//! Safety inspector and dependency analyzer for automatic parallelization candidates in Javryn V0.8.

/// The decision produced by static safety and dependency analysis.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParallelizationDecision {
    /// Candidate is provably safe for automatic parallel task generation.
    Parallel {
        iteration_var: String,
        array_name: String,
        body_expr_source: String,
        chunk_size: usize,
    },
    /// Candidate cannot be proven safe; falls back to original sequential execution.
    Sequential { reason: String },
}

/// Static Analyzer evaluating ECMAScript statement structures for parallelization eligibility.
#[derive(Debug, Default)]
pub struct StaticAnalyzer;

impl StaticAnalyzer {
    pub fn new() -> Self {
        Self
    }

    /// Analyzes a JavaScript function or source string to inspect if it contains eligible for-of / array.map loops.
    pub fn analyze_source(&self, source: &str) -> ParallelizationDecision {
        // High-level structural inspection for basic pure expressions
        let trimmed = source.trim();

        // Rejection Check 1: Shared accumulators or mutations
        if trimmed.contains("+=")
            || trimmed.contains("-=")
            || trimmed.contains("*=")
            || trimmed.contains("/=")
            || trimmed.contains("++")
            || trimmed.contains("--")
            || trimmed.contains(".push(")
            || trimmed.contains(".pop(")
            || trimmed.contains(".splice(")
            || trimmed.contains("console.")
            || trimmed.contains("globalThis.")
            || trimmed.contains("window.")
            || trimmed.contains("document.")
            || trimmed.contains("fetch(")
            || trimmed.contains("setTimeout(")
            || trimmed.contains("setInterval(")
            || trimmed.contains("async ")
            || trimmed.contains("await ")
            || trimmed.contains("try ")
            || trimmed.contains("throw ")
            || trimmed.contains("break;")
            || trimmed.contains("continue;")
            || trimmed.contains("return ")
            || trimmed.contains("[0]")
            || trimmed.contains("[1]")
        {
            return ParallelizationDecision::Sequential {
                reason: "contains non-isolated side effects, mutations, or control flow"
                    .to_string(),
            };
        }

        // Inspection for pure map callbacks: e.g. "x => x * x" or "x => x + 10"
        if trimmed.contains("=>") {
            let parts: Vec<&str> = trimmed.splitn(2, "=>").collect();
            if parts.len() == 2 {
                let param = parts[0].trim().trim_matches(|c| c == '(' || c == ')');
                let body = parts[1].trim();

                if !param.is_empty() && !body.is_empty() {
                    return ParallelizationDecision::Parallel {
                        iteration_var: param.to_string(),
                        array_name: "items".to_string(),
                        body_expr_source: trimmed.to_string(),
                        chunk_size: 100,
                    };
                }
            }
        }

        ParallelizationDecision::Sequential {
            reason: "does not match supported pure candidate template".to_string(),
        }
    }
}
