use crate::ast::{Error, Game, Type};
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

const RESERVED_SYMBOLS: [&str; 6] = ["begin", "end", "goals", "player", "score", "visible"];
const RESERVED_TYPES: [&str; 6] = [
    "Bool",
    "Goals",
    "Player",
    "PlayerOrKeeper",
    "Score",
    "Visibility",
];

impl Game<Arc<str>> {
    pub fn mangle_symbols(&mut self) -> Result<(), Error<Arc<str>>> {
        // Do not mangle reserved symbols.
        let mut hashed: BTreeMap<_, _> = RESERVED_SYMBOLS
            .into_iter()
            .map(|symbol| (Arc::from(symbol), Arc::from(symbol)))
            .collect();

        // Do not mangle reserved types and their values.
        for identifier in RESERVED_TYPES {
            hashed.insert(Arc::from(identifier), Arc::from(identifier));
            let score = self.resolve_typedef_or_fail(&Arc::from(identifier))?;
            if let Type::Set { identifiers, .. } = score.type_.as_ref() {
                for identifier in identifiers {
                    hashed.insert(identifier.clone(), identifier.clone());
                }
            }
        }

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

#[cfg(test)]
mod test {
    use crate::test_transform;

    test_transform!(
        mangle_symbols,
        preserve_types,
        "
        type Bool = { 0, 1 };
        type Goals = Player -> Score;
        type Int = { 0, 1, 2 };
        type Player = { x };
        type PlayerOrKeeper = { x, keeper };
        type Score = { 5 };
        type Visibility = Player -> Bool;

        const dec: Int -> Int = { 2: 1, :0 };

        var goals: Goals = { :5 };
        var player: PlayerOrKeeper = keeper;
        var v: Int = 2;
        var visible: Visibility = { :1 };

        begin, loop: player = x;
        loop, loop: ;
        loop, l: $ a;
        loop, r: $ a;
        l, set: v != 0;
        r, set: v != 0;
        set, loop: v = dec[v];
        loop, end: player = keeper;
        ",
        "
        type Bool = { 0, 1 };
        type Goals = Player -> Score;
        type _j = { 0, 1, _k };
        type Player = { x };
        type PlayerOrKeeper = { x, keeper };
        type Score = { 5 };
        type Visibility = Player -> Bool;

        const _i: _j -> _j = { _k: 1, :0 };

        var goals: Goals = { :5 };
        var player: PlayerOrKeeper = keeper;
        var _p: _j = _k;
        var visible: Visibility = { :1 };

        begin, _l: player = x;
        _l, _l: ;
        _l, _n: $ _m;
        _l, _o: $ _m;
        _n, _q: _p != 0;
        _o, _q: _p != 0;
        _q, _l: _p = _i[_p];
        _l, end: player = keeper;
        "
    );
}
