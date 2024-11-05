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
                &mut edge.label
            {
                game.simplify_expression(lhs, &mut new_constants);
                game.simplify_expression(rhs, &mut new_constants);
            }
        }

        self.constants
            .extend(new_constants.into_iter().map(|(constant, _, _)| constant));

        Ok(())
    }

    fn maybe_merge_access(
        &self,
        outer: &Expression<Id>,
        inner: &Expression<Id>,
        new_constants: &mut Vec<NewConstant>,
    ) -> Option<(Id, Arc<Expression<Id>>)> {
        let (Expression::Reference { identifier }, Expression::Access { lhs, rhs, .. }) =
            (outer.uncast(), inner.uncast())
        else {
            return None;
        };
        let outer_id = identifier;
        let inner_id = lhs.uncast().as_reference()?;
        if let Some((constant, _, _)) = new_constants
            .iter()
            .find(|(_, outer, inner)| outer == outer_id && inner == inner_id)
        {
            return Some((constant.identifier.clone(), rhs.clone()));
        }
        let outer_const = &self.resolve_constant(outer_id).or_else(|| {
            new_constants
                .iter()
                .find(|(c, _, _)| c.identifier == *outer_id)
                .map(|(c, _, _)| c)
        })?;
        let inner_const = &self.resolve_constant(inner_id).or_else(|| {
            new_constants
                .iter()
                .find(|(c, _, _)| c.identifier == *inner_id)
                .map(|(c, _, _)| c)
        })?;
        let value = merge_values(&outer_const.value, &inner_const.value)?;
        let fresh_name: Arc<str> = Id::from(format!("__gen_{outer_id}_{inner_id}"));
        let type_ = {
            let outer_type_ = outer_const.type_.resolve(self).ok()?;
            let inner_type_ = inner_const.type_.resolve(self).ok()?;
            let (Type::Arrow { rhs, .. }, Type::Arrow { lhs, .. }) = (outer_type_, inner_type_)
            else {
                return None;
            };
            Some(Arc::new(Type::Arrow {
                lhs: lhs.clone(),
                rhs: rhs.clone(),
            }))
        }?;
        let new_constant = Constant::new(Span::none(), fresh_name.clone(), type_, Arc::new(value));
        new_constants.push((new_constant, outer_id.clone(), inner_id.clone()));
        Some((fresh_name, rhs.clone()))
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
                if let Some((identifier, new_rhs)) =
                    self.maybe_merge_access(lhs, rhs, new_constants)
                {
                    *lhs = Arc::new(Expression::new(identifier));
                    *rhs = new_rhs;
                }
            }
            Expression::Cast { rhs, .. } => self.simplify_expression(rhs, new_constants),
            Expression::Reference { .. } => {}
        }
    }
}

// Create a new map: keys are from inner_map, values are from outer_map
fn merge_values(outer_map: &Value<Id>, inner_map: &Value<Id>) -> Option<Value<Id>> {
    let (Value::Map { .. }, Value::Map { entries, .. }) = (outer_map, inner_map) else {
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
}
