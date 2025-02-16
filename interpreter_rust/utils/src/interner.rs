use std::collections::BTreeMap;
use std::fmt::Debug;

pub struct Interner<Key, Value> {
    string_to_id: BTreeMap<Key, Value>,
}

impl<Key, Value> Default for Interner<Key, Value> {
    fn default() -> Self {
        Self {
            string_to_id: BTreeMap::new(),
        }
    }
}

impl<Key: Clone + Ord, Value: Clone + PartialEq + TryFrom<usize>> Interner<Key, Value> {
    pub fn intern(&mut self, string: &Key) -> Value
    where
        <Value as TryFrom<usize>>::Error: Debug,
    {
        const ERROR: &str = "Maximum number of interned keys reached! Increase Value size.";
        if let Some(id) = self.string_to_id.get(string) {
            return id.clone();
        }

        let id = self
            .string_to_id
            .len()
            .checked_add(1)
            .expect(ERROR)
            .try_into()
            .expect(ERROR);
        self.intern_as(string, id)
    }

    pub fn intern_as(&mut self, string: &Key, id: Value) -> Value {
        assert!(!self.string_to_id.contains_key(string));
        self.string_to_id.insert(string.clone(), id.clone());
        id
    }

    pub fn interned(&self, string: &Key) -> Option<&Value> {
        self.string_to_id.get(string)
    }

    pub fn recall(&self, id: &Value) -> Option<&Key> {
        self.string_to_id
            .iter()
            .find(|pair| pair.1 == id)
            .map(|pair| pair.0)
    }
}
