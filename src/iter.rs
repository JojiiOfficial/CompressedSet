use crate::{item::Item, CompressedSequence};

/// Iterator over a CompressedSequence
pub struct CompSeqIterRef<'a> {
    set: &'a CompressedSequence,
    pos: usize,
    ipos: usize,
    item: Option<&'a Item>,
}

impl<'a> CompSeqIterRef<'a> {
    #[inline]
    pub fn new(set: &'a CompressedSequence) -> Self {
        let item = set.seq().get(0);
        Self {
            set,
            pos: 0,
            ipos: 0,
            item,
        }
    }
}

impl<'a> Iterator for CompSeqIterRef<'a> {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let curr_item = self.item?;

            if let Some(val) = curr_item.at(self.ipos, self.set.step) {
                self.ipos += 1;
                return Some(val);
            }

            self.ipos = 0;
            self.pos += 1;
            self.item = self.set.seq().get(self.pos);
        }
    }
}

/// Iterator over a CompressedSequence
pub struct CompSeqIter {
    set: CompressedSequence,
    pos: usize,
    ipos: usize,
    item: Option<Item>,
}

impl CompSeqIter {
    #[inline]
    pub fn new(mut set: CompressedSequence) -> Self {
        let item = set.seq_mut().pop();
        Self {
            set,
            pos: 0,
            ipos: 0,
            item,
        }
    }
}

impl Iterator for CompSeqIter {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let curr_item = self.item?;

            if let Some(val) = curr_item.at(self.ipos, self.set.step) {
                self.ipos += 1;
                return Some(val);
            }

            self.ipos = 0;
            self.pos += 1;
            self.item = self.set.seq_mut().pop();
        }
    }
}
