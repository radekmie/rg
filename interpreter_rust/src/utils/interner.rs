use std::collections::BTreeMap;
use std::convert::{TryFrom, TryInto};
use std::fmt::Debug;

#[derive(Default)]
pub struct Interner<Id: Ord> {
    // TODO: Maybe `Rc<String>` would be better?
    id_to_string: BTreeMap<Id, String>,
    string_to_id: BTreeMap<String, Id>,
}

impl<Id: Copy + Ord + TryFrom<usize>> Interner<Id> {
    pub fn intern(&mut self, string: &str) -> Id
    where
        <Id as TryFrom<usize>>::Error: Debug,
    {
        if let Some(id) = self.string_to_id.get(string) {
            return *id;
        }

        const ERROR: &str = "Maximum number of interned strings reached! Increase Id size.";
        let id = self
            .string_to_id
            .len()
            .checked_add(1)
            .expect(ERROR)
            .try_into()
            .expect(ERROR);
        self.intern_as(string, id)
    }

    pub fn intern_as(&mut self, string: &str, id: Id) -> Id {
        assert!(!self.id_to_string.contains_key(&id));
        assert!(!self.string_to_id.contains_key(string));
        self.id_to_string.insert(id, string.into());
        self.string_to_id.insert(string.into(), id);
        id
    }

    pub fn recall(&self, id: &Id) -> Option<&String> {
        self.id_to_string.get(id)
    }
}
