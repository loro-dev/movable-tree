use std::{collections::BTreeMap, fmt::Debug};

/// This is an implementation of [log-spaced-snapshots](https://madebyevan.com/algos/log-spaced-snapshots/)
///
/// If we have n versions, this algorithm would only keep around `2^d * log(n)` snapshots, where d is the custom parameter.
///
/// When n = 1M:
///
/// ```log
/// | d | Number of snapshots |
/// |:- | :------------------ |
/// | 1 | 21                  |
/// | 2 | 40                  |
/// | 3 | 76                  |
/// | 4 | 144                 |
/// ```
///
/// K is a custom version type that should be `Ord`, and V is the value type.
///
/// # Example
///
/// ```no_run
/// use movable_tree::log_spaced_snapshots::LogSpacedSnapshots;
/// let mut cache: LogSpacedSnapshots<usize, usize> = LogSpacedSnapshots::new(3);
/// for i in 0..10000 {
///     cache.push(i, i);
/// }
/// let (v, s) = cache.pop_till_snapshot_lte(&9999).unwrap();
/// assert_eq!(*v, 9999);
/// assert_eq!(*s, 9999);
/// let (v, s) = cache.pop_till_snapshot_lte(&9998).unwrap();
/// assert_eq!(*v, 9998);
/// assert_eq!(*s, 9998);
/// let (v, s) = cache.pop_till_snapshot_lte(&6000).unwrap();
/// assert_eq!(*v, 5119);
/// assert_eq!(*s, 5119);
/// assert!(cache.pop_till_snapshot_lte(&2000).is_none());
/// ```
///
///
#[derive(Debug, Clone)]
pub struct LogSpacedSnapshots<K, V> {
    keys: Vec<K>,
    cache: BTreeMap<usize, V>,
    d: usize,
}

impl<K, T> LogSpacedSnapshots<K, T> {
    pub fn new(d: usize) -> Self {
        Self {
            keys: Default::default(),
            cache: Default::default(),
            d,
        }
    }
}

impl<K: Ord, T> LogSpacedSnapshots<K, T> {
    /// Push a new snapshot.
    /// The new version must be greatest version.
    pub fn push(&mut self, version: K, value: T) {
        let d = self.d;
        let new_version = self.keys.len();
        let delta = first_zero_bit(new_version) << d;
        if new_version >= delta {
            self.cache.remove(&(new_version - delta));
        }
        self.cache.insert(new_version, value);
        if let Some(last) = self.keys.last() {
            assert!(&version > last);
        }
        self.keys.push(version);
    }

    /// Pop the history until the latest snapshot's version <= k
    pub fn pop_till_snapshot_lte(&mut self, k: &K) -> Option<(&K, &T)> {
        let first_to_remove = match self.keys.binary_search(k) {
            Ok(n) => n + 1,
            Err(n) => n,
        };
        self.cache.retain(|&k, _| k < first_to_remove);
        match self.cache.last_key_value() {
            Some((key, _)) => {
                self.keys.drain(key + 1..);
            }
            None => {
                self.keys.clear();
            }
        }

        self.keys
            .last()
            .map(|k| (k, self.cache.last_key_value().unwrap().1))
    }

    pub fn cache_size(&self) -> usize {
        self.cache.len()
    }
}

impl<K, T> Default for LogSpacedSnapshots<K, T> {
    fn default() -> Self {
        Self::new(2)
    }
}

fn first_zero_bit(n: usize) -> usize {
    (n + 1) & !n
}

#[cfg(test)]
mod test {
    use super::*;
    // fn test_d(d: usize, n: usize) -> usize {
    //     let mut count = 0;
    //     let mut set: HashSet<usize> = Default::default();
    //     for i in 0..n {
    //         set.insert(i);
    //         if i > (first_zero_bit(i) << d) {
    //             set.remove(&(i - (first_zero_bit(i) << d)));
    //         } else {
    //             count += 1;
    //         }
    //         assert_eq!(set.len(), count);
    //     }

    //     set.len()
    // }
    // #[test]
    // fn t() {
    //     println!("{:b}", first_zero_bit(0b1010));
    //     println!("{:b}", first_zero_bit(0b1011));
    //     println!("{:b}", first_zero_bit(0b1111));

    //     println!("{}", test_d(1, 1_000_000));
    //     println!("{}", test_d(2, 1_000_000));
    //     println!("{}", test_d(3, 1_000_000));
    //     println!("{}", test_d(4, 1_000_000));
    // }

    #[test]
    fn snapshot() {
        let mut cache: LogSpacedSnapshots<usize, usize> = LogSpacedSnapshots::new(3);
        for i in 0..10000 {
            cache.push(i, i);
        }
        let (v, s) = cache.pop_till_snapshot_lte(&9999).unwrap();
        assert_eq!(*v, 9999);
        assert_eq!(*s, 9999);
        let (v, s) = cache.pop_till_snapshot_lte(&9998).unwrap();
        assert_eq!(*v, 9998);
        assert_eq!(*s, 9998);
        let (v, s) = cache.pop_till_snapshot_lte(&6000).unwrap();
        assert_eq!(*v, 5119);
        assert_eq!(*s, 5119);
        assert!(cache.pop_till_snapshot_lte(&2000).is_none());
    }
}
