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
            match &mut edge.label {
                Label::Assignment { lhs, rhs } | Label::Comparison { lhs, rhs, .. } => {
                    simplify_expression(lhs, &game, &mut new_constants);
                    simplify_expression(rhs, &game, &mut new_constants);
                }
                _ => {}
            }
        }

        let new_constants = new_constants.into_iter().map(|(constant, _, _)| constant);
        self.constants.extend(new_constants);

        Ok(())
    }

    fn maybe_merge_access(
        &self,
        outer: &Expression<Id>,
        inner: &Expression<Id>,
        new_constants: &mut Vec<NewConstant>,
    ) -> Option<(Id, Arc<Expression<Id>>)> {
        match (outer.uncast(), inner.uncast()) {
            (Expression::Reference { identifier }, Expression::Access { lhs, rhs, .. }) => {
                let outer_id = identifier;
                let inner_id = lhs.uncast().as_reference()?;
                if let Some((constant, _, _)) = new_constants
                    .iter()
                    .find(|(_, outer, inner)| outer == outer_id && inner == inner_id)
                {
                    return Some((constant.identifier.clone(), rhs.clone()));
                }
                let outer_const = &self.resolve_constant(outer_id)?;
                let inner_const = &self.resolve_constant(inner_id)?;
                merge_values(&outer_const.value, &inner_const.value).and_then(|value| {
                    let fresh_name: Arc<str> = Arc::from(format!("__gen_{outer_id}_{inner_id}"));
                    let tpe = {
                        let outer_tpe = outer_const.type_.resolve(self);
                        let inner_tpe = inner_const.type_.resolve(self);
                        match (outer_tpe, inner_tpe) {
                            (Ok(Type::Arrow { rhs, .. }), Ok(Type::Arrow { lhs, .. })) => {
                                Some(Arc::new(Type::Arrow {
                                    lhs: lhs.clone(),
                                    rhs: rhs.clone(),
                                }))
                            }
                            _ => None,
                        }
                    }?;
                    let new_constant =
                        Constant::new(Span::none(), fresh_name.clone(), tpe, Arc::new(value));
                    new_constants.push((new_constant, outer_id.clone(), inner_id.clone()));
                    Some((fresh_name, rhs.clone()))
                })
            }
            _ => None,
        }
    }

    fn simplify_expression(
        &self,
        expr: &Expression<Id>,
        new_constants: &mut Vec<NewConstant>,
    ) -> Option<Expression<Id>> {
        match expr {
            Expression::Access { lhs, rhs, .. } => self
                .maybe_merge_access(lhs, rhs, new_constants)
                .map(|(identifier, rhs)| Expression::Access {
                    span: Span::none(),
                    lhs: Arc::new(Expression::new(identifier)),
                    rhs,
                })
                .or_else(|| {
                    let new_lhs = self.simplify_expression(lhs, new_constants).map(Arc::new);
                    let new_rhs = self.simplify_expression(rhs, new_constants).map(Arc::new);
                    if new_lhs.is_none() && new_rhs.is_none() {
                        None
                    } else {
                        Some(Expression::Access {
                            span: Span::none(),
                            lhs: new_lhs.unwrap_or_else(|| lhs.clone()),
                            rhs: new_rhs.unwrap_or_else(|| rhs.clone()),
                        })
                    }
                }),
            Expression::Cast { span, lhs, rhs } => self
                .simplify_expression(rhs, new_constants)
                .map(|rhs| Expression::Cast {
                    span: *span,
                    lhs: lhs.clone(),
                    rhs: Arc::new(rhs),
                }),
            Expression::Reference { .. } => None,
        }
    }
}

// Create a new map: keys are from inner_map, values are from outer_map
fn merge_values(outer_map: &Value<Id>, inner_map: &Value<Id>) -> Option<Value<Id>> {
    let (Value::Map { .. }, Value::Map { entries, .. }) = (outer_map, inner_map) else {
        return None;
    };
    let entries: Option<Vec<ValueEntry<Id>>> = entries
        .iter()
        .map(
            |ValueEntry {
                 identifier, value, ..
             }| {
                let new_value = outer_map.get_entry(value.to_identifier()?)?;
                Some(ValueEntry::new(
                    Span::none(),
                    identifier.clone(),
                    Arc::new(new_value.clone()),
                ))
            },
        )
        .collect();
    entries.map(|entries| Value::Map {
        span: Span::none(),
        entries,
    })
}

fn simplify_expression(
    expr: &mut Arc<Expression<Id>>,
    game: &Game<Id>,
    new_constants: &mut Vec<NewConstant>,
) {
    while let Some(new_expr) = game.simplify_expression(expr, new_constants) {
        *expr = Arc::new(new_expr);
    }
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
        const __gen_MapCC_MapBC: B -> C = { 1: 3, 2: 1, :1 };
        const __gen_MapAB_MapAA: A -> B = { 1: 1, 2: 2, :2 };
        var x: C = 1;
        begin, a1: x = __gen_MapBC_MapAB[1];
        begin, a2: x = __gen_MapBC_MapAB[2];
        begin, b: x = __gen_MapCC_MapCC[1];
        begin, c: x = __gen_MapCC_MapBC[MapAB[1]];
        begin, d: x = MapCC[VarMap[1]];
        begin, e: x = __gen_MapCC_MapCC[1];
        begin, f: x = Cast(__gen_MapCC_MapCC[1]);
        begin, g: x = __gen_MapCC_MapCC[__gen_MapCC_MapCC[1]];
        begin, h: x = __gen_MapCC_MapBC[__gen_MapAB_MapAA[1]];
        begin, i: x = MapCC[VarMap[__gen_MapAB_MapAA[1]]];
        begin, j: x = __gen_MapCC_MapBC[1][VarMap[1]];"
    );
}
