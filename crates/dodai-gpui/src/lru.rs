//! A small least-recently-used map for caches of GPU images: when it is full, inserting drops the
//! entry used longest ago and hands it back, so the caller can free its texture
//! (`App::drop_image`) instead of leaving it in the sprite atlas.

use std::collections::HashMap;
use std::hash::Hash;

/// At most `limit` entries.
pub(crate) struct Lru<K, V> {
    map: HashMap<K, (V, u64)>,
    tick: u64,
    limit: usize,
}

impl<K: Hash + Eq + Copy, V: Clone> Lru<K, V> {
    /// An empty map holding at most `limit` entries (at least one).
    pub(crate) fn new(limit: usize) -> Self {
        Self {
            map: HashMap::new(),
            tick: 0,
            limit: limit.max(1),
        }
    }

    /// The value under `key`, marked as just used.
    pub(crate) fn get(&mut self, key: K) -> Option<V> {
        self.tick += 1;
        let tick = self.tick;
        self.map.get_mut(&key).map(|(v, used)| {
            *used = tick;
            v.clone()
        })
    }

    /// Store `value` under `key`; returns the values pushed out (the replaced one and, when
    /// full, the least recently used).
    pub(crate) fn insert(&mut self, key: K, value: V) -> Vec<V> {
        self.tick += 1;
        let mut out: Vec<V> = self
            .map
            .insert(key, (value, self.tick))
            .map(|(v, _)| v)
            .into_iter()
            .collect();
        while self.map.len() > self.limit {
            let Some(oldest) = self
                .map
                .iter()
                .min_by_key(|(_, (_, used))| *used)
                .map(|(k, _)| *k)
            else {
                break;
            };
            if let Some((v, _)) = self.map.remove(&oldest) {
                out.push(v);
            }
        }
        out
    }

    /// Number of entries.
    #[cfg(test)]
    pub(crate) fn len(&self) -> usize {
        self.map.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evicts_the_least_recently_used_and_returns_it() {
        let mut c = Lru::new(2);
        assert!(c.insert(1, "a").is_empty());
        assert!(c.insert(2, "b").is_empty());
        assert_eq!(c.get(1), Some("a"));
        assert_eq!(c.insert(3, "c"), vec!["b"]);
        assert_eq!(c.len(), 2);
        assert_eq!(c.get(2), None);
        assert_eq!(c.get(1), Some("a"));
    }

    #[test]
    fn replacing_a_key_hands_back_the_old_value() {
        let mut c = Lru::new(4);
        c.insert(7, 1);
        assert_eq!(c.insert(7, 2), vec![1]);
        assert_eq!(c.get(7), Some(2));
        assert_eq!(c.len(), 1);
    }

    #[test]
    fn a_zero_limit_still_keeps_the_newest() {
        let mut c = Lru::new(0);
        c.insert(1, 'x');
        assert_eq!(c.insert(2, 'y'), vec!['x']);
        assert_eq!(c.get(2), Some('y'));
    }
}
