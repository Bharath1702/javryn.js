//! Chunk planner for automatic parallelization in Javryn V0.8.

/// Calculates dynamic chunk sizing based on total items and available physical workers.
pub fn calculate_chunk_size(total_items: usize, worker_count: usize) -> usize {
    if total_items <= 100 || worker_count == 0 {
        return total_items.max(1);
    }

    let target_chunks_per_worker = 4;
    let desired_chunks = worker_count * target_chunks_per_worker;
    let calculated = total_items.div_ceil(desired_chunks);

    calculated.clamp(25, 1000)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_chunk_size_small() {
        assert_eq!(calculate_chunk_size(50, 4), 50);
    }

    #[test]
    fn test_calculate_chunk_size_large() {
        let chunk = calculate_chunk_size(10000, 4);
        assert!((25..=1000).contains(&chunk));
    }
}
