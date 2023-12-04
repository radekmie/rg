use crate::ast::{Error, Game};
use map_id::MapId;
use std::collections::BTreeMap;
use std::str::from_utf8;
use std::sync::Arc;

const ALPHABET: &[u8] = "0123456789abcdefghijklmnopqrstuvwxyz".as_bytes();
fn base36(mut input: usize) -> Arc<str> {
    let mut index = 2 + input.ilog(ALPHABET.len()) as usize;
    let mut chars = vec![b'_'; index];
    while input > 0 {
        index -= 1;
        chars[index] = ALPHABET[input % ALPHABET.len()];
        input /= ALPHABET.len();
    }

    Arc::from(from_utf8(&chars).unwrap())
}

const RESERVED_SYMBOLS: &[&str] = &[
    "0",
    "1",
    "Bool",
    "Goals",
    "Player",
    "PlayerOrKeeper",
    "Score",
    "Visibility",
    "begin",
    "end",
    "goals",
    "keeper",
    "player",
    "score",
    "visible",
];

impl Game<Arc<str>> {
    pub fn mangle_symbols(&mut self) -> Result<(), Error<Arc<str>>> {
        let mut hashed: BTreeMap<_, _> = RESERVED_SYMBOLS
            .iter()
            .map(|symbol| (Arc::from(*symbol), Arc::from(*symbol)))
            .collect();

        *self = self.map_id(&mut |id| {
            let next = hashed.len() + 1;
            hashed
                .entry(id.clone())
                .or_insert_with(|| base36(next))
                .clone()
        });

        Ok(())
    }
}
