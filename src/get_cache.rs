pub struct GetCache {
    // (pos, vec_pos, len)
    cache: Vec<(u32, u32, u32)>,
}

impl GetCache {
    #[inline]
    pub fn new() -> Self {
        Self { cache: vec![] }
    }

    #[inline]
    pub fn with_capacity(size: usize) -> Self {
        Self {
            cache: Vec::with_capacity(size),
        }
    }

    #[inline]
    pub fn insert(&mut self, pos: u32, vec_pos: u32, len: u32) {
        match self.cache.binary_search_by(|i| i.0.cmp(&pos)) {
            Ok(_) => return,
            Err(new_pos) => {
                self.cache.insert(new_pos, (pos, vec_pos, len));
            }
        }
    }

    pub fn get(&self, pos: u32) -> Option<(u32, u32, u32)> {
        match self.cache.binary_search_by(|i| i.0.cmp(&pos)) {
            Ok(cpos) => Some(self.cache[cpos]),
            Err(tpos) => (tpos > 0).then(|| self.cache[tpos - 1]),
        }
    }
}

#[cfg(test)]
mod test {
    use super::GetCache;

    #[test]
    fn test_cache_insert() {
        let mut get_cache = GetCache::new();

        get_cache.insert(10, 12, 0);
        get_cache.insert(4, 6, 0);
        get_cache.insert(5, 7, 0);
        get_cache.insert(90, 8, 0);

        let items: Vec<_> = get_cache.cache.iter().map(|i| i.0).collect();
        let mut exp_items = items.clone();
        exp_items.sort_unstable();
        assert_eq!(items, exp_items);
    }

    #[test]
    fn test_cache_get() {
        let mut get_cache = GetCache::new();

        get_cache.insert(10, 7, 6);
        get_cache.insert(4, 2, 6);
        get_cache.insert(5, 4, 6);
        get_cache.insert(90, 30, 6);

        assert_eq!(get_cache.get(12), Some((10, 12, 6)));
        assert_eq!(get_cache.get(13), Some((10, 12, 6)));
        assert_eq!(get_cache.get(1239085), Some((90, 8, 6)));
    }
}
