pub mod get_cache;
pub mod item;
pub mod iter;
pub mod utils;

use get_cache::GetCache;
use item::Item;
use iter::{CompSeqIter, CompSeqIterRef};
use serde::{Deserialize, Serialize};
use std::{cmp::Ordering, mem::size_of, num::NonZeroU16};

/// A compressed sequence of numbers somewhat near to each other
/// with a frequently occurring step size
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompressedSequence {
    step: u32,
    seq: Vec<Item>,
}

impl std::fmt::Debug for CompressedSequence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let number_items = self.seq.iter().filter(|i| i.is_numbers()).count();
        let seq_items = self.seq.iter().filter(|i| i.is_sequence()).count();

        let half_numbers = self
            .seq
            .iter()
            .filter(|i| i.is_numbers())
            .filter(|i| {
                if let Item::Numbers(_, b) = i {
                    return b.is_none();
                }
                false
            })
            .count();

        f.debug_struct("CompressedSequence")
            .field("steps", &self.step)
            .field("seq_len", &self.seq.len())
            .field("> numbers", &number_items)
            .field("> numbers (half)", &half_numbers)
            .field("> sequences", &seq_items)
            .field(
                "> bytes size",
                &(self.size_of() as f32 / 10240.0f32.powi(2)),
            )
            .field(
                "> raw bytes size",
                &((self.len() as f32 * 4.0) / 1024.0f32.powi(2)),
            )
            .finish()
    }
}

impl CompressedSequence {
    /// Create a new compressed sequence with a given step size
    #[inline]
    pub fn new(step: u32) -> Self {
        Self { seq: vec![], step }
    }

    /// Creates a new compressed sequence from an iterator
    pub fn from_iterator<I>(step: u32, iter: I) -> Self
    where
        I: IntoIterator<Item = u32>,
    {
        let mut vec: Vec<u32> = iter.into_iter().collect();
        vec.sort_unstable();
        let mut seq = Self::new(step);
        seq.extend(vec);
        seq
    }

    /// Pushes a new value to the sequence
    ///
    /// # Panics
    /// panics if the same item was pushed twice
    pub fn push(&mut self, item: u32) {
        if self.seq.is_empty() {
            self.seq.push(Item::new(item));
            return;
        }

        let step_size = self.step;

        if !self.seq.last().unwrap().can_add() {
            self.seq.push(Item::new(item));
            return;
        }

        let last_nr = self.last_item().unwrap().last_number(step_size);

        if last_nr + step_size == item {
            let mut seq = self.seq.pop().unwrap().to_sequence();
            seq.seq_add();
            self.seq.push(seq);
            return;
        }

        let last_item = self.last_item_mut().unwrap();
        if let Item::Numbers(nr, next) = last_item {
            assert!(next.is_none());
            if item == *nr {
                panic!("Can't push the same value twice");
            }

            if item <= *nr || item - *nr > u16::MAX as u32 {
                self.seq.push(Item::new(item));
                return;
            }

            *next = Some(NonZeroU16::new((item - *nr) as u16).unwrap());
            return;
        }

        self.seq.push(Item::new(item));
    }

    /// Copies the data to a newly allocated Vec<u32>
    pub fn to_vec(&self) -> Vec<u32> {
        let mut out = vec![];

        for item in self.seq.iter() {
            match item {
                Item::Numbers(start, next) => {
                    out.push(*start);
                    if let Some(next) = next {
                        out.push(*start + next.get() as u32);
                    }
                }
                Item::Sequence(start, count) => {
                    for i in 0..*count as u32 + 1 {
                        out.push(*start + self.step * i);
                    }
                }
            }
        }

        out
    }

    /// Gets an item at the given position
    pub fn get(&self, pos: usize) -> Option<u32> {
        let mut item: Option<&Item> = None;
        let mut i_len = 0;

        for i in self.seq.iter() {
            let next_len = i_len + i.len();
            if pos < next_len {
                item = Some(i);
                break;
            }

            i_len = next_len;
        }

        let item = item?;
        Some(item.at(pos - i_len, self.step)?)
    }

    /// Gets an item at the given position using position cache for more efficient lookups
    pub fn get_cached(&self, pos: usize, cache: &mut GetCache) -> Option<u32> {
        let mut item: Option<&Item> = None;
        let mut i_len = 0;

        let mut vec_pos = 0;
        let mut skip = 0;

        if let Some((_, vec_pos, itemlen)) = cache.get(pos as u32) {
            item = Some(&self.seq[vec_pos as usize]);
            i_len = itemlen as usize;
            skip = vec_pos as usize;
        }

        for (p, i) in self.seq.iter().enumerate().skip(skip) {
            let next_len = i_len + i.len();
            if pos < next_len {
                vec_pos = p as u32;
                item = Some(i);
                break;
            }

            i_len = next_len;
        }

        let item = item?;
        let item_value = item.at(pos - i_len, self.step)?;
        cache.insert(pos as u32, vec_pos, i_len as u32);

        Some(item_value)
    }

    /// Returns `true` if the set contains the given item using binary search
    pub fn has_bin_search(&self, item: u32) -> bool {
        let mut size = self.len();
        let mut left = 0;
        let mut right = size;

        let mut pos_cache = GetCache::with_capacity(16);

        while left < right {
            let mid = left + size / 2;

            let cmp = self.get_cached(mid, &mut pos_cache).unwrap().cmp(&item);

            if cmp == Ordering::Less {
                left = mid + 1;
            } else if cmp == Ordering::Greater {
                right = mid;
            } else {
                return true;
            }

            size = right - left;
        }

        false
    }

    /// Searches the set in linear time for the given `item`
    #[inline]
    pub fn contains(&self, item: u32) -> bool {
        self.iter().any(|i| i == item)
    }

    /// Returns the length of compressed set
    pub fn len(&self) -> usize {
        let mut len = 0;

        for item in self.seq.iter() {
            match item {
                Item::Numbers(_, next) => {
                    len += 1;
                    if next.is_some() {
                        len += 1;
                    }
                }
                Item::Sequence(_, count) => {
                    len += *count as usize + 1;
                }
            }
        }

        len
    }

    /// Returns `true` if there is no value in the set
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.seq.is_empty()
    }

    /// Returns the size of the set in bytes
    #[inline]
    pub fn size_of(&self) -> usize {
        let size_self = size_of::<Self>();
        size_self + self.seq.len() * size_of::<Item>()
    }

    /// Returns an iterator over all items in the set
    #[inline]
    pub fn iter(&self) -> CompSeqIterRef {
        CompSeqIterRef::new(self)
    }

    #[inline]
    pub(crate) fn seq(&self) -> &Vec<Item> {
        &self.seq
    }

    #[inline]
    pub(crate) fn seq_mut(&mut self) -> &mut Vec<Item> {
        &mut self.seq
    }

    #[inline]
    fn last_item(&self) -> Option<&Item> {
        self.seq.last()
    }

    #[inline]
    fn last_item_mut(&mut self) -> Option<&mut Item> {
        self.seq.last_mut()
    }
}

impl IntoIterator for CompressedSequence {
    type Item = u32;

    type IntoIter = CompSeqIter;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        CompSeqIter::new(self)
    }
}

impl Extend<u32> for CompressedSequence {
    #[inline]
    fn extend<T: IntoIterator<Item = u32>>(&mut self, iter: T) {
        for i in iter {
            self.push(i);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_push() {
        let mut comp_seq = CompressedSequence::new(10);

        let mut exp = vec![];

        for j in 1..30u32 {
            for i in (j * 1..=j * 100).step_by(10) {
                comp_seq.push(i);
                exp.push(i);
            }

            for i in (j * 100..=j * 120).step_by(3) {
                comp_seq.push(i);
                exp.push(i);
            }

            for i in (j * 150..=j * 200).step_by(10) {
                comp_seq.push(i);
                exp.push(i);
            }
        }

        println!("{:#?}", comp_seq);
        assert_eq!(comp_seq.to_vec(), exp);
        assert_eq!(comp_seq.len(), exp.len());

        for (pos, i) in exp.iter().enumerate() {
            assert_eq!(comp_seq.get(pos), Some(*i));
        }
    }

    #[test]
    fn test_bin_search() {
        let mut comp_seq = CompressedSequence::new(10);

        let mut exp = vec![];

        for (pos, i) in (0..=9120).step_by(10).enumerate() {
            comp_seq.push(i);
            exp.push(i);

            if pos % 42 == 0 {
                comp_seq.push(i + 1);
                exp.push(i + 1);
            }
        }

        assert_eq!(comp_seq.to_vec(), exp);
        assert_eq!(comp_seq.len(), exp.len());

        for (pos, i) in exp.iter().enumerate() {
            assert_eq!(comp_seq.get(pos), Some(*i));
            assert!(comp_seq.has_bin_search(*i));
        }
    }

    #[test]
    fn test_iter() {
        let mut comp_seq = CompressedSequence::new(10);

        let mut exp = vec![];

        for (pos, i) in (0..=9120).step_by(10).enumerate() {
            comp_seq.push(i);
            exp.push(i);

            if pos % 42 == 0 {
                comp_seq.push(i + 1);
                exp.push(i + 1);
            }
        }

        assert_eq!(comp_seq.iter().collect::<Vec<_>>(), exp);
    }

    #[test]
    fn test_big_sequence() {
        let mut comp_seq = CompressedSequence::new(10);
        for i in (0..1_000_000).step_by(10) {
            comp_seq.push(i);
        }

        assert_eq!(comp_seq.len(), 100_000);
    }

    #[test]
    fn test_get_smol() {
        let comp_seq = CompressedSequence::new(10);
        assert_eq!(comp_seq.get(0), None);
    }
}
