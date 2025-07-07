use crate::ast::{Constant, Error, Expression, Game, Label, Type, Value, ValueEntry};
use std::sync::Arc;
use utils::position::Span;

type Id = Arc<str>;
type NewConstant = (Constant<Id>, Id, Id);

impl Game<Id> {
    pub fn merge_accesses(&mut self) -> Result<(), Error<Id>> {
        let mut new_constants = vec![];
        let game = Self {
            constants: self.constants.clone(),
            typedefs: self.typedefs.clone(),
            ..Self::default()
        };
        for edge in &mut self.edges {
            if let Label::Assignment { lhs, rhs } | Label::Comparison { lhs, rhs, .. } =
                &mut Arc::make_mut(edge).label
            {
                game.simplify_expression(lhs, &mut new_constants);
                game.simplify_expression(rhs, &mut new_constants);
            }
        }

        for (constant, _, _) in new_constants {
            if !self.constants.contains(&constant) {
                self.constants.push(constant);
            }
        }

        Ok(())
    }

    fn maybe_merge_access(
        &self,
        outer: &Expression<Id>,
        inner: &Expression<Id>,
        new_constants: &mut Vec<NewConstant>,
    ) -> Option<(Id, Vec<Arc<Expression<Id>>>)> {
        if !inner.uncast().is_access() {
            return None;
        }

        let (outer_id, mut indexes) = outer.access_identifier_with_accessors();
        let is_first_arg = indexes.is_empty();
        let (inner_id, mut inner_indexes) = inner.access_identifier_with_accessors();
        indexes.append(&mut inner_indexes);

        if let Some((constant, _, _)) = new_constants
            .iter()
            .find(|(_, outer, inner)| outer == outer_id && inner == inner_id)
        {
            return Some((
                constant.identifier.clone(),
                indexes.into_iter().cloned().collect(),
            ));
        }
        let outer_const = self.resolve_new_constant(outer_id, new_constants)?;
        let inner_const = self.resolve_new_constant(inner_id, new_constants)?;
        let mut outer_type = outer_const.type_.clone();
        Arc::make_mut(&mut outer_type).resolve_recursive(self);
        let mut inner_type = inner_const.type_.clone();
        Arc::make_mut(&mut inner_type).resolve_recursive(self);

        let mut outer_value = outer_const.value.clone();
        Arc::make_mut(&mut outer_value).resolve_recursive(self);
        let mut inner_value = inner_const.value.clone();
        Arc::make_mut(&mut inner_value).resolve_recursive(self);
        let new_type = create_new_type(&outer_type, &inner_type, is_first_arg)?;

        let value = {
            match (outer.uncast(), inner.uncast()) {
                // mapA[X][mapB[Y]]
                (
                    Expression::Access { lhs: outer_lhs, .. },
                    Expression::Access { lhs: inner_lhs, .. },
                ) if inner_lhs.uncast().is_reference() && outer_lhs.uncast().is_reference() => {
                    merge_maps_a_x_b_y(&outer_value, &inner_value)
                }
                // mapA[mapB[Y][Z]]
                (Expression::Reference { .. }, Expression::Access { lhs: inner_lhs, .. })
                    if inner_lhs.uncast().is_access() =>
                {
                    match inner_lhs.uncast() {
                        Expression::Access { lhs, .. } if lhs.uncast().is_reference() => {
                            merge_maps_a_b_y_z(&outer_value, &inner_value)
                        }
                        _ => None,
                    }
                }
                // mapA[mapB[X]]
                (Expression::Reference { .. }, Expression::Access { lhs, .. })
                    if lhs.uncast().is_reference() =>
                {
                    merge_values(&outer_value, &inner_value)
                }
                _ => None,
            }
        }?;

        let fresh_name: Arc<str> = Id::from(format!("__gen_{outer_id}_{inner_id}"));
        let new_constant = Constant::new(
            Span::none(),
            fresh_name.clone(),
            Arc::new(new_type),
            Arc::new(value),
        );
        new_constants.push((new_constant, outer_id.clone(), inner_id.clone()));
        Some((fresh_name, indexes.into_iter().cloned().collect()))
    }

    fn simplify_expression(
        &self,
        expr: &mut Arc<Expression<Id>>,
        new_constants: &mut Vec<NewConstant>,
    ) {
        match Arc::make_mut(expr) {
            Expression::Access { lhs, rhs, .. } => {
                self.simplify_expression(lhs, new_constants);
                self.simplify_expression(rhs, new_constants);
                if let Some((identifier, indexes)) =
                    self.maybe_merge_access(lhs, rhs, new_constants)
                {
                    let new_expr = create_access(identifier, indexes);
                    *expr = Arc::new(new_expr);
                }
            }
            Expression::Cast { rhs, .. } => self.simplify_expression(rhs, new_constants),
            Expression::Reference { .. } => {}
        }
    }

    fn resolve_new_constant<'a>(
        &'a self,
        id: &Id,
        new_constants: &'a [NewConstant],
    ) -> Option<&'a Constant<Id>> {
        self.resolve_constant(id).or_else(|| {
            new_constants
                .iter()
                .find(|(c, _, _)| c.identifier == *id)
                .map(|(c, _, _)| c)
        })
    }
}

fn create_access(identifier: Id, indexes: Vec<Arc<Expression<Id>>>) -> Expression<Id> {
    indexes
        .into_iter()
        .fold(Expression::new(identifier), |acc, index| {
            Expression::Access {
                span: Span::none(),
                lhs: Arc::new(acc),
                rhs: index,
            }
        })
}

// both types are dealiased
fn create_new_type(
    outer_type: &Type<Id>,
    inner_type: &Type<Id>,
    is_first_arg: bool,
) -> Option<Type<Id>> {
    let (
        Type::Arrow {
            lhs: outer_lhs,
            rhs: outer_rhs,
        },
        Type::Arrow {
            lhs: inner_lhs,
            rhs: inner_rhs,
        },
    ) = (outer_type, inner_type)
    else {
        return None;
    };

    match (outer_rhs.as_ref(), inner_rhs.as_ref()) {
        // mapA[X][mapB[Y]]
        // mapA : C -> (B -> D)
        // mapB: A -> B
        // mapAB[X][Y]
        // mapAB: C -> (A -> D)
        (Type::Arrow { rhs, .. }, _)
            if !rhs.is_arrow() && !inner_rhs.is_arrow() && !is_first_arg =>
        {
            Some(Type::Arrow {
                lhs: outer_lhs.clone(),
                rhs: Arc::new(Type::Arrow {
                    lhs: inner_lhs.clone(),
                    rhs: rhs.clone(),
                }),
            })
        }
        // mapA[mapB[Y][Z]]
        // mapA: C -> D
        // mapB: A -> (B -> C)
        // mapAB[Y][Z]
        // mapAB: A -> (B -> D)
        (_, Type::Arrow { lhs, rhs }) if !outer_rhs.is_arrow() && !rhs.is_arrow() => {
            Some(Type::Arrow {
                lhs: inner_lhs.clone(),
                rhs: Arc::new(Type::Arrow {
                    lhs: lhs.clone(),
                    rhs: outer_rhs.clone(),
                }),
            })
        }
        // mapA[mapB[X]]
        // mapA : B -> C
        // mapB : A -> B
        // mapAB[X]
        // mapAB : A -> C
        (_, _) if !outer_rhs.is_arrow() && !inner_rhs.is_arrow() => Some(Type::Arrow {
            lhs: inner_lhs.clone(),
            rhs: outer_rhs.clone(),
        }),
        _ => None,
    }
}

/// For case mapA[mapB[Y][Z]]
/// where mapA is a map from C -> D and mapB is a map from A to (B -> C),
/// we need to merge the maps such mapAB[Y][Z] becomes A -> (B -> D)
fn merge_maps_a_b_y_z(outer_map: &Value<Id>, inner_map: &Value<Id>) -> Option<Value<Id>> {
    let (
        Value::Map { .. },
        Value::Map {
            entries: y_entries, ..
        },
    ) = (outer_map, inner_map)
    else {
        return None;
    };

    let new_entries: Option<Vec<_>> = y_entries
        .iter()
        .map(|entry| {
            let new_value = merge_values(outer_map, &entry.value)?;
            Some(ValueEntry::new(
                Span::none(),
                entry.identifier.clone(),
                Arc::new(new_value.clone()),
            ))
        })
        .collect();

    let new_entries = new_entries?;

    Some(Value::Map {
        span: Span::none(),
        entries: new_entries,
    })
}

/// For case mapA[X][mapB[Y]]
/// where mapA is a map from C -> (B -> D) and mapB is a map from A to B,
/// we need to merge the maps such mapAB[X][Y] becomes C -> (A -> D)
fn merge_maps_a_x_b_y(outer_map: &Value<Id>, inner_map: &Value<Id>) -> Option<Value<Id>> {
    let Value::Map { entries, .. } = outer_map else {
        return None;
    };

    let new_entries: Option<Vec<_>> = entries
        .iter()
        .map(|entry| {
            let new_value = merge_values(&entry.value, inner_map)?;
            Some(ValueEntry::new(
                Span::none(),
                entry.identifier.clone(),
                Arc::new(new_value),
            ))
        })
        .collect();

    let new_entries = new_entries?;
    Some(Value::Map {
        span: Span::none(),
        entries: new_entries,
    })
}

/// Create a new map: keys are from `inner_map`, values are from `outer_map`.
fn merge_values(outer_map: &Value<Id>, inner_map: &Value<Id>) -> Option<Value<Id>> {
    let Value::Map { entries, .. } = inner_map else {
        return None;
    };
    let entries: Option<_> = entries
        .iter()
        .map(|entry| {
            Some(ValueEntry::new(
                Span::none(),
                entry.identifier.clone(),
                Arc::new(outer_map.get_entry(entry.value.to_identifier()?)?.clone()),
            ))
        })
        .collect();
    let mut entries: Vec<_> = entries?;
    let default_value = entries
        .iter()
        .find(|entry| entry.identifier.is_none())?
        .value
        .clone();
    // Remove entries that map to default value
    entries.retain(|entry| entry.identifier.is_none() || entry.value != default_value);

    Some(Value::Map {
        span: Span::none(),
        entries,
    })
}

#[cfg(test)]
mod test {
    use crate::test_transform;

    test_transform!(
        merge_accesses,
        small,
        "type A = {1,2,3,4};
        type B = {1,2};
        type C = {1,2,3};
        const MapAB: A -> B = {1: 1, :2};
        const MapAA: A -> A = {1: 1, 2:2, :3};
        const MapBC: B -> C = {1: 2, 2: 3, :4};
        const MapCC: C -> C = {1: 2, 2: 3, :1};
        var x: C = 1;
        begin, a1: x = MapBC[MapAB[1]];
        begin, a2: x = MapBC[MapAB[2]];
        begin, b: x = MapCC[MapCC[1]];
        begin, c: x = MapCC[MapBC[MapAB[1]]];
        begin, d: x = MapCC[VarMap[1]];
        begin, e: x = MapCC[Cast(MapCC[1])];
        begin, f: x = Cast(MapCC[MapCC[1]]);
        begin, g: x = MapCC[MapCC[MapCC[MapCC[1]]]];
        begin, h: x = MapCC[MapBC[MapAB[MapAA[1]]]];
        begin, i: x = MapCC[VarMap[MapAB[MapAA[1]]]];
        begin, j: x = MapCC[MapBC[1]][VarMap[1]];",
        "type A = { 1, 2, 3, 4 };
        type B = { 1, 2 };
        type C = { 1, 2, 3 };
        const MapAB: A -> B = { 1: 1, :2 };
        const MapAA: A -> A = { 1: 1, 2: 2, :3 };
        const MapBC: B -> C = { 1: 2, 2: 3, :4 };
        const MapCC: C -> C = { 1: 2, 2: 3, :1 };
        const __gen_MapBC_MapAB: A -> C = { 1: 2, :3 };
        const __gen_MapCC_MapCC: C -> C = { 1: 3, 2: 1, :2 };
        const __gen_MapCC___gen_MapBC_MapAB: A -> C = { 1: 3, :1 };
        const __gen_MapCC___gen_MapCC_MapCC: C -> C = { 1: 1, 2: 2, :3 };
        const __gen_MapCC___gen_MapCC___gen_MapCC_MapCC: C -> C = { 1: 2, 2: 3, :1 };
        const __gen_MapAB_MapAA: A -> B = { 1: 1, :2 };
        const __gen_MapBC___gen_MapAB_MapAA: A -> C = { 1: 2, :3 };
        const __gen_MapCC___gen_MapBC___gen_MapAB_MapAA: A -> C = { 1: 3, :1 };
        const __gen_MapCC_MapBC: B -> C = { 1: 3, :1 };
        var x: C = 1;
        begin, a1: x = __gen_MapBC_MapAB[1];
        begin, a2: x = __gen_MapBC_MapAB[2];
        begin, b: x = __gen_MapCC_MapCC[1];
        begin, c: x = __gen_MapCC___gen_MapBC_MapAB[1];
        begin, d: x = MapCC[VarMap[1]];
        begin, e: x = __gen_MapCC_MapCC[1];
        begin, f: x = Cast(__gen_MapCC_MapCC[1]);
        begin, g: x = __gen_MapCC___gen_MapCC___gen_MapCC_MapCC[1];
        begin, h: x = __gen_MapCC___gen_MapBC___gen_MapAB_MapAA[1];
        begin, i: x = MapCC[VarMap[__gen_MapAB_MapAA[1]]];
        begin, j: x = __gen_MapCC_MapBC[1][VarMap[1]];"
    );

    test_transform!(
        merge_accesses,
        complex,
        "type AA = A -> A;
        type Dir = {u, d};
        type DirAA = Dir -> AA;
        const up: A -> A = {1:2, 2:3, :4};
        const down: A -> A = {:1, 3:2, 4:3};
        const uup: AA = up;
        const inDir: Dir -> AA = {u: uup, :down};
        const inDirr: DirAA = inDir;
        a, b: up[inDir[u][1]] == 1;
        b, c: inDir[u][uup[1]] == 1;
        c, d: uup[up[A(1)]] == A(1);
        a, b: up[A(inDir[u][1])] == 1;
        b, c: inDirr[u][A(up[1])] == 1;
        c, d: up[uup[1]] == A(1);
        a, b: inDir[u][inDir[u][up[1]]] == 1;",
        "type AA = A -> A;
        type Dir = { u, d };
        type DirAA = Dir -> AA;
        const up: A -> A = { 1: 2, 2: 3, :4 };
        const down: A -> A = { :1, 3: 2, 4: 3 };
        const uup: AA = up;
        const inDir: Dir -> AA = { u: uup, :down };
        const inDirr: DirAA = inDir;
        const __gen_up_inDir: Dir -> A -> A = { u: { 1: 3, :4 }, :{ :2, 3: 3, 4: 4 } };
        const __gen_inDir_uup: Dir -> A -> A = { u: { 1: 3, :4 }, :{ 1: 1, 2: 2, :3 } };
        const __gen_uup_up: A -> A = { 1: 3, :4 };
        const __gen_inDirr_up: Dir -> A -> A = { u: { 1: 3, :4 }, :{ 1: 1, 2: 2, :3 } };
        const __gen_up_uup: A -> A = { 1: 3, :4 };
        const __gen_inDir_up: Dir -> A -> A = { u: { 1: 3, :4 }, :{ 1: 1, 2: 2, :3 } };
        a, b: __gen_up_inDir[u][1] == 1;
        b, c: __gen_inDir_uup[u][1] == 1;
        c, d: __gen_uup_up[A(1)] == A(1);
        a, b: __gen_up_inDir[u][1] == 1;
        b, c: __gen_inDirr_up[u][1] == 1;
        c, d: __gen_up_uup[1] == A(1);
        a, b: inDir[u][__gen_inDir_up[u][1]] == 1;"
    );

    test_transform!(
        merge_accesses,
        too_complex,
        "type AA = A -> A;
        type AAA = A -> AA;
        type Dir = {u, d};
        const up: A -> A = {1:2, 2:3, :4};
        const down: A -> A = {:1, 3:2, 4:3};
        const double: A -> AA = { :{:0}};
        const triple: A -> A -> AA = {:{:{:0}}};
        const tripled: A -> AAA = triple;
        const inDir: Dir -> AA = {u: up, :down};
        // a, b: triple[1][1][up[1]] == 1;
        b, c: double[up[1]][1] == 1;
        // c, d: tripled[1][1][up[1]] == 1;
        // a, b: inDir[u][triple[1][1][up[1]]] == 1;
        // a, b: inDir[u][triple[1][1][1]] == 1;
        "
    );
}
