use alloc::vec::Vec;

#[derive(Debug)]
pub struct DoubleVecIndex<K: core::hash::Hash + Clone, V: Clone + Into<K>> {
    values: Vec<V>,
    keys: Vec<K>,
}

impl<K: core::hash::Hash + Clone + Eq, V: Clone + Into<K>>
    DoubleVecIndex<K, V>
{
    pub fn new(values: Vec<V>) -> Self {
        let mut keys = Vec::new();
        for value in values.clone() {
            keys.push(value.into());
        }
        Self { values, keys }
    }
    pub fn take(&mut self, key: K) -> Option<V> {
        for (i, k) in self.keys.iter().enumerate() {
            if key == k.clone() {
                self.keys.remove(i);
                let out = self.values.remove(i);
                return Some(out);
            }
        }
        None
    }
    pub fn keys(&self) -> Vec<K> {
        self.keys.clone()
    }
}
