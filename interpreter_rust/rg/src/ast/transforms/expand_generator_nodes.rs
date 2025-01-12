use crate::ast::{Error, Game, Node, Pragma};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

type Id = Arc<str>;

impl Game<Id> {
    pub fn expand_generator_nodes(&mut self) -> Result<(), Error<Id>> {
        for index in (0..self.edges.len()).rev() {
            let bindings = self.edges[index].bindings();
            if !bindings.is_empty() {
                let mappings = self.create_mappings(bindings.into_iter())?;
                let item = self.edges.remove(index);
                self.edges.splice(
                    index..index,
                    mappings
                        .into_iter()
                        .map(|mapping| item.substitute_bindings(&mapping))
                        .map(Arc::from),
                );
            }
        }

        // First, split pragmas into multiple with distinctive bindings. Thanks
        // to that, the next step does not suffer from combinatorial explosion.
        for index in (0..self.pragmas.len()).rev() {
            if self.pragmas[index].has_bindings() {
                let pragmas = self.pragmas.remove(index).split_by_bindings();
                self.pragmas.splice(index..index, pragmas);
            }
        }

        for index in (0..self.pragmas.len()).rev() {
            let bindings = self.pragmas[index].bindings();
            if !bindings.is_empty() {
                let mappings = self.create_mappings(bindings.into_iter())?;
                if let Some(pragmas) = self.pragmas[index].substitute_bindings_mut(&mappings) {
                    self.pragmas.splice(index..index + 1, pragmas);
                }
            }
        }

        Ok(())
    }
}

impl Pragma<Id> {
    fn split_by_bindings(self) -> Vec<Self> {
        match self {
            Self::Disjoint { span, node, nodes } => group_by_bindings(nodes)
                .map(|nodes| Self::Disjoint {
                    span,
                    node: node.clone(),
                    nodes,
                })
                .collect(),
            Self::DisjointExhaustive { span, node, nodes } => group_by_bindings(nodes)
                .map(|nodes| Self::DisjointExhaustive {
                    span,
                    node: node.clone(),
                    nodes,
                })
                .collect(),
            Self::Repeat {
                span,
                nodes,
                identifiers,
            } => group_by_bindings(nodes)
                .map(|nodes| Self::Repeat {
                    span,
                    nodes,
                    identifiers: identifiers.clone(),
                })
                .collect(),
            // `@simpleApply{,Exhaustive}` cannot have binds in thier nodes.
            Self::SimpleApply { .. } | Self::SimpleApplyExhaustive { .. } => vec![self.clone()],
            Self::TagIndex { span, index, nodes } => group_by_bindings(nodes)
                .map(|nodes| Self::TagIndex { span, index, nodes })
                .collect(),
            Self::TagMaxIndex { span, index, nodes } => group_by_bindings(nodes)
                .map(|nodes| Self::TagMaxIndex { span, index, nodes })
                .collect(),
            Self::TranslatedFromRbg { .. } => vec![self.clone()],
            Self::Unique { span, nodes } => group_by_bindings(nodes)
                .map(|nodes| Self::Unique { span, nodes })
                .collect(),
        }
    }
}

fn group_by_bindings(nodes: Vec<Node<Id>>) -> impl Iterator<Item = Vec<Node<Id>>> {
    nodes
        .into_iter()
        .fold(
            BTreeMap::new(),
            |mut grouped: BTreeMap<BTreeSet<_>, Vec<_>>, node| {
                let bindings = node
                    .bindings()
                    .map(|(x, y)| (x.clone(), y.clone()))
                    .collect();
                grouped.entry(bindings).or_default().push(node);
                grouped
            },
        )
        .into_values()
}

#[cfg(test)]
mod test {
    use crate::test_transform;

    test_transform!(
        expand_generator_nodes,
        edge,
        "type T = { a, b }; x(t: T), y: ;",
        "type T = { a, b }; x__bind__a, y: ; x__bind__b, y: ;"
    );

    test_transform!(
        expand_generator_nodes,
        pragma1,
        "type T = { a, b }; @unique x(t: T);",
        "type T = { a, b }; @unique x__bind__a x__bind__b;"
    );

    test_transform!(expand_generator_nodes,
        pragma2,
        "type T = { a, b }; @unique y(t1: T)(t2: T);",
        "type T = { a, b }; @unique y__bind__a__bind__a y__bind__a__bind__b y__bind__b__bind__a y__bind__b__bind__b;"
    );

    test_transform!(
        expand_generator_nodes,
        pragma3,
        "type T = { a, b }; @simpleApply x y(t: T) [t: T];",
        "type T = { a, b }; @simpleApply x y__bind__a [a]; @simpleApply x y__bind__b [b];"
    );

    test_transform!(
        expand_generator_nodes,
        pragma4,
        "type T = { a, b }; var v: T = a; @simpleApply x y(t: T) [t: T] v = t;",
        "type T = { a, b }; var v: T = a; @simpleApply x y__bind__a [a] v = T(a); @simpleApply x y__bind__b [b] v = T(b);"
    );

    test_transform!(
        expand_generator_nodes,
        pragma5,
        "type T1 = { a, b }; type T2 = { a, b }; @unique x1(t1: T1) x2(t2: T2);",
        "type T1 = { a, b }; type T2 = { a, b }; @unique x1__bind__a x1__bind__b; @unique x2__bind__a x2__bind__b;"
    );
}
