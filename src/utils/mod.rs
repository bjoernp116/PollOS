use alloc::vec::Vec;




#[derive(Debug)]
pub struct DoubleVecIndex<K, V> {
    values: Vec<V>,
    keys: Vec<K>
}

impl<K: From<V> + Eq, V: Clone> DoubleVecIndex<K, V> {
    pub fn new(values: Vec<V>) -> Self {
        let mut keys = Vec::new();
        for value in values.clone() {
            keys.push(value.into());
        }
        Self {
            values,
            keys,
        }
    }
    pub fn take(&mut self, key: K) -> Option<V> {
        for (i, k) in self.keys.iter().enumerate() {
            if key == *k {
                self.keys.remove(i);
                return Some(self.values.remove(i));
            }
        }
        None
    }
}
