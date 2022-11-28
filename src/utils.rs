use std::collections::HashMap;

/// Returns the most frequent step size in the given set
#[inline]
pub fn max_step_size(list: &[u32]) -> u32 {
    get_steps_freq(list)
        .iter()
        .max_by(|a, b| a.1.cmp(&b.1))
        .map(|i| *i.0)
        .unwrap_or(1)
}

/// Returns the most frequent step sizes in the given set
pub fn get_steps_freq(list: &[u32]) -> HashMap<u32, u32> {
    let mut step_size_freq: HashMap<u32, u32> = HashMap::new();
    for (b, a) in list.iter().zip(list.iter().skip(1)) {
        *step_size_freq.entry(a - b).or_default() += 1;
    }
    step_size_freq
}
