extern crate smallvec;

use std::mem;
use std::iter::{FromIterator, Extend};
use smallvec::{SmallVec, Array};

// TODO:
// We store redundant `indices` here for better complexity. It might be good to extract this to a
// seperate struct to allow us to be generic over whether we store only `lengths` or both fields.
// The extra wasted space is fine for my use-case but others might not have that luxury.
//
// This would look like: having a `LengthOnly(Vec<usize>)` and
// `LengthAndIndices(Vec<usize>, Vec<usize>)` struct, then implementing `GetNthLength` and
// `GetNthIndex` traits. In the `LengthOnly` case we can calculate it each time. If we do this it
// would also be good to use `VecLike` for all of the fields (`elements` included).
pub struct JaggedArray<Element, A: Array<Item = usize> = [usize; 8]> {
    elements: Vec<Element>,
    lengths: SmallVec<A>,
    indices: SmallVec<A>,
}

pub struct Iter<'a, Element: 'a> {
    elements: &'a [Element],
    lengths: &'a [usize],
}

impl<'a, Element> Iterator for Iter<'a, Element> {
    type Item = &'a [Element];

    // TODO: We can trust all of this - do it unsafely
    fn next(&mut self) -> Option<Self::Item> {
        if self.lengths.is_empty() {
            return None;
        }

        let (now_lens, rest_lens) = self.lengths.split_at(1);
        let now_len = now_lens[0];
        let (now_el, rest_el) = self.elements.split_at(now_len);

        self.lengths = rest_lens;
        self.elements = rest_el;

        Some(now_el)
    }
}

pub struct IterMut<'a, Element: 'a> {
    elements: &'a mut [Element],
    lengths: &'a [usize],
}

impl<'a, Element> Iterator for IterMut<'a, Element> {
    type Item = &'a mut [Element];

    // TODO: We can trust all of this - do it unsafely
    fn next(&mut self) -> Option<Self::Item> {
        if self.lengths.is_empty() {
            return None;
        }

        let (now_lens, rest_lens) = self.lengths.split_at(1);
        let now_len = now_lens[0];
        let (now_el, rest_el) = mem::replace(&mut self.elements, &mut []).split_at_mut(now_len);

        self.lengths = rest_lens;
        self.elements = rest_el;

        Some(now_el)
    }
}

impl<Element, A: Array<Item = usize>> Default for JaggedArray<Element, A> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, Element, A: Array<Item = usize>> IntoIterator for &'a JaggedArray<Element, A> {
    type Item = <Self::IntoIter as Iterator>::Item;
    type IntoIter = Iter<'a, Element>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, Element, A: Array<Item = usize>> IntoIterator for &'a mut JaggedArray<Element, A> {
    type Item = <Self::IntoIter as Iterator>::Item;
    type IntoIter = IterMut<'a, Element>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<Element, A: Array<Item = usize>> JaggedArray<Element, A> {
    pub fn new() -> Self {
        JaggedArray {
            elements: Default::default(),
            lengths: Default::default(),
            indices: Default::default(),
        }
    }

    pub fn iter(&self) -> Iter<Element> {
        Iter {
            elements: &self.elements,
            lengths: &self.lengths,
        }
    }

    pub fn iter_mut(&mut self) -> IterMut<Element> {
        IterMut {
            elements: &mut self.elements,
            lengths: &mut self.lengths,
        }
    }

    pub fn get(&self, n: usize) -> Option<&[Element]> {
        self.lengths
            .get(n)
            .and_then(|len| self.indices.get(n).map(|index| (*len, *index)))
            .map(|(len, index)| &self.elements[index..index + len])
    }

    pub fn get_mut(&mut self, n: usize) -> Option<&mut [Element]> {
        // Explicit if let instead of `.map` to prevent borrowck errors
        if let Some((len, index)) =
            self.lengths
                .get(n)
                .and_then(|len| self.indices.get(n).map(|index| (*len, *index))) {
            Some(&mut self.elements[index..index + len])
        } else {
            None
        }
    }
}

impl<Element: Clone, A: Array<Item = usize>> JaggedArray<Element, A> {
    pub fn push(&mut self, slice: &[Element]) {
        self.lengths.push(slice.len());
        let new_index = self.elements.len();
        self.indices.push(new_index);
        self.elements.extend_from_slice(slice);
    }
}

impl<Element: Clone, A: Array<Item = usize>, Slice: AsRef<[Element]>> Extend<Slice>
    for JaggedArray<Element, A> {
    fn extend<It: IntoIterator<Item = Slice>>(&mut self, iterator: It) {
        let mut total_length: usize = self.elements.len();

        for slice in iterator {
            let slice: &[Element] = slice.as_ref();
            let len = slice.len();

            self.lengths.push(len);
            self.indices.push(total_length);
            self.elements.extend_from_slice(slice);

            total_length += len;
        }
    }
}

impl<Element: Clone, A: Array<Item = usize>, Slice: AsRef<[Element]>> FromIterator<Slice>
    for JaggedArray<Element, A> {
    fn from_iter<It: IntoIterator<Item = Slice>>(iterator: It) -> Self {
        let mut out: Self = Default::default();
        out.extend(iterator);
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assert_returns_same_elements() {
        let input = vec![vec![1, 2, 3, 4, 5], vec![2, 3, 4], vec![2; 5]];

        let jagged: JaggedArray<_> = input.iter().collect();

        let output: Vec<Vec<_>> = jagged.iter().map(|slice| slice.to_owned()).collect();

        assert_eq!(input, output);
    }
}
