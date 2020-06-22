use std::ops::{Index, IndexMut};
pub struct ExpandingVector<T> {
    values: Vec<Option<T>>,
}

impl<T: Clone> ExpandingVector<T> {
    pub fn new() -> ExpandingVector<T> {
        ExpandingVector { values: vec![None] }
    }
}

impl<T: Clone> Index<usize> for ExpandingVector<T> {
    type Output = Option<T>;

    fn index(&self, index: usize) -> &Option<T> {
        if index >= self.values.len() {
            &None
        } else {
            &self.values[index]
        }
    }
}

impl<T: Clone> IndexMut<usize> for ExpandingVector<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        if index >= self.values.len() {
            assert!(index < self.values.len() * 2);
            self.values.resize(self.values.len() * 2, None)
        }

        &mut self.values[index]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_expanding_vec() {
        let mut v = ExpandingVector::new();

        v[0] = Some(9);

        assert_eq!(Some(9), v[0]);
        assert_eq!(None, v[1]);
        assert_eq!(None, v[2]);
        assert_eq!(None, v[2000]);

        v[1] = Some(5);
        assert_eq!(Some(9), v[0]);
        assert_eq!(Some(5), v[1]);
        assert_eq!(None, v[2000]);

        v[3] = Some(8);
        assert_eq!(Some(9), v[0]);
        assert_eq!(Some(5), v[1]);
        assert_eq!(Some(8), v[3]);
    }
}
