//! The `multi_map` module represents an implementation of a `MultiMap` to be used by the
//! Redis module system.
use std::collections::hash_map::Iter;
use std::collections::HashMap;
use std::iter::IntoIterator;

/// `MultiMap` represents a map of `String` keys to a `Vec` of `String` values.
#[derive(Clone, Debug, Default)]
pub struct MultiMap {
    inner: HashMap<String, Vec<String>>,
}

impl MultiMap {
    /// New returns an initialized `MultiMap`.
    pub fn new() -> MultiMap {
        MultiMap {
            inner: HashMap::new(),
        }
    }

    /// Insert will add all values passed as an argument to the specified key.
    pub fn insert<K, I>(&mut self, key: K, values: I)
    where
        K: Into<String>,
        I: IntoIterator<Item = String>,
    {
        let entry = self.inner.entry(key.into()).or_insert_with(|| vec![]);
        values.into_iter().for_each(|item| entry.push(item));
    }

    /// Given a key, return the length of the values.
    pub fn key_len<K: Into<String>>(&self, key: K) -> usize {
        let values = self.inner.get(&key.into());
        match values {
            Some(v) => v.len(),
            None => 0,
        }
    }

    /// Given a key, return a list of the values.
    pub fn values<K: Into<String>>(&self, key: K) -> Option<&Vec<String>> {
        self.inner.get(&key.into())
    }

    /// Given a key, remove it from the map. A return value of 0 indicates that the key didn't
    /// priorly exist while 1 means it was successfully removed.
    pub fn delete_key<K: Into<String>>(&mut self, key: K) -> i64 {
        const ONE: i64 = 1;
        const ZERO: i64 = 0;
        let del_k = &key.into();

        match self.inner.get(del_k) {
            Some(_) => {
                self.inner.remove(del_k);
                ONE
            }
            None => ZERO,
        }
    }

    /// Return the number of pairs in the map.
    pub fn len(&self) -> usize {
        self.inner.len()
    }
}

impl<'a> IntoIterator for &'a MultiMap {
    type Item = (&'a String, &'a Vec<String>);
    type IntoIter = Iter<'a, String, Vec<String>>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.iter()
    }
}
