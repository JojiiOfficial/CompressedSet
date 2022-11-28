use serde::{Deserialize, Serialize};
use std::num::NonZeroU16;

/// A number item within a set
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum Item {
    /// Number and successor
    Numbers(u32, Option<NonZeroU16>),
    /// Sequence of numbers in format (starting, steps)
    Sequence(u32, u16),
}

impl Item {
    /// Create a new simple number value item
    #[inline]
    pub fn new(v: u32) -> Self {
        Self::Numbers(v, None)
    }

    /// Adds the next value to the sequence
    #[inline]
    pub fn seq_add(&mut self) {
        if let Self::Sequence(_, cnt) = self {
            *cnt += 1;
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        match self {
            Item::Numbers(_, b) => {
                if b.is_some() {
                    2
                } else {
                    1
                }
            }
            Item::Sequence(_, slen) => *slen as usize + 1,
        }
    }

    #[inline]
    pub fn at(&self, pos: usize, step_size: u32) -> Option<u32> {
        match self {
            Item::Numbers(a, b) => {
                if pos == 0 {
                    Some(*a)
                } else if pos == 1 {
                    Some(a + (*b)?.get() as u32)
                } else {
                    None
                }
            }
            Item::Sequence(start, cnt) => {
                if pos > *cnt as usize {
                    return None;
                }
                Some(start + (pos as u32 * step_size))
            }
        }
    }

    /// Returns the last number of an item
    #[inline]
    pub fn last_number(&self, step_size: u32) -> u32 {
        match self {
            Item::Numbers(a, b) => {
                if let Some(b) = b {
                    a + b.get() as u32
                } else {
                    *a
                }
            }
            Item::Sequence(start, count) => start + (step_size * *count as u32),
        }
    }

    /// Converts the item to a sequence. This only works for numbers
    /// without a successor
    #[inline]
    pub fn to_sequence(self) -> Self {
        match self {
            Item::Numbers(a, b) => {
                if b.is_some() {
                    panic!("Can't convert two numbers to sequence");
                }

                Self::Sequence(a, 0)
            }
            Item::Sequence(_, _) => self,
        }
    }

    /// Returns `true` if there can be a value added
    #[inline]
    pub fn can_add(&self) -> bool {
        match self {
            Item::Numbers(_, b) => b.is_none(),
            Item::Sequence(_, cnt) => *cnt != u16::MAX,
        }
    }

    /// Returns `true` if the item is [`Numbers`].
    ///
    /// [`Numbers`]: Item::Numbers
    #[must_use]
    pub(crate) fn is_numbers(&self) -> bool {
        matches!(self, Self::Numbers(..))
    }

    /// Returns `true` if the item is [`Sequence`].
    ///
    /// [`Sequence`]: Item::Sequence
    #[must_use]
    pub(crate) fn is_sequence(&self) -> bool {
        matches!(self, Self::Sequence(..))
    }
}
