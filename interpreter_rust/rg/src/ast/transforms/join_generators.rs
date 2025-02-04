use std::sync::Arc;

use crate::ast::{Error, Game};

type Id = Arc<str>;

impl Game<Id> {
    pub fn join_generators(&mut self) -> Result<(), Error<Id>> {
        todo!()
    }
}

#[cfg(test)]
mod test {
    use crate::test_transform;

    test_transform!(
        join_generators,
        small1,
        "type All = {1,2,3,4,5,6};
        type Type1 = {3,4,5};
        type Type2 = {1,2,3};
        type Type3 = {1,3,6};
        var coord : All = 1;
        begin, a1(t1: Type1): coord == All(1);
        begin, a2(t2: Type2): coord == All(2);
        begin, a3(t3: Type3): coord == All(3);
        a1(t1: Type1), end: coord = t1;
        a2(t2: Type2), end: coord = t2;
        a3(t3: Type3), end: coord = t3;",
        "type All = { 1, 2, 3, 4, 5, 6 };
        type Type1 = { 3, 4, 5 };
        type Type2 = { 1, 2, 3 };
        type Type3 = { 1, 3, 6 };
        type Joined_1 = { 1, 2, 3, 4, 5, 6 };
        const joined_1: All -> Joined_1 -> Bool = { 1: { 3: 1, 4: 1, 5: 1, :0 }, 2: { 1: 1, 2: 1, 3: 1, :0 }, 3: { 1: 1, 3: 1, 6: 1, :0 }, :{ :0 } };
        var coord: All = 1;
        begin, 1(bind_Joined_1: Joined_1): joined_1[coord][bind_Joined_1] == 1;
        1(bind_Joined_1: Joined_1), end: coord = bind_Joined_1;"
    );

    test_transform!(
        join_generators,
        small2,
        "type All = {1,2,3,4,5,6};
        type Type1 = {3,4,5};
        var coord : All = 1;
        begin, a1(t1: Type1): coord == All(1);
        begin, f: coord == 1;
        f, e: coord = 2;
        a1(t1: Type1), end: coord = t1;"
    );

    test_transform!(
        join_generators,
        small3,
        "type All = {1,2,3,4,5,6};
        type Type1 = {3,4,5};
        var coord : All = 1;
        begin, a1(t1: Type1): coord == All(1);
        begin, f: ;
        a1(t1: Type3), end: coord = t1;"
    );

    test_transform!(
        join_generators,
        small4,
        "type All = {1,2,3,4,5,6};
        type Type1 = {3,4,5};
        type Type2 = {1,2,3};
        type Type3 = {1,3,6};
        var coord : All = 1;
        begin, a1(t1: Type1): coord == All(1);
        begin, a2(t2: Type2): coord == All(2);
        begin, a2(t2: Type2): ;
        begin, a3(t3: Type3): coord == All(3);
        a1(t1: Type1), end: coord = t1;
        a2(t2: Type2), end: coord = t2;
        a3(t3: Type3), end: coord = t3;"
    );
}
