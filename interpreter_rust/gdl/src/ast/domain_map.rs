use crate::ast::domain_graph::{DomainGraph, DomainGraphNode};
use crate::ast::{AtomOrVariable, Predicate, Rule, Term};
use std::collections::{BTreeMap, BTreeSet};

pub type DomainMap<Id> = BTreeMap<Id, BTreeSet<Id>>;

impl<Id: Clone + Ord> AtomOrVariable<Id> {
    fn domain_map(
        &self,
        domain_map: &mut DomainMap<Id>,
        domain_graph: &DomainGraph<Id>,
        domain_graph_node: Option<DomainGraphNode<Id>>,
    ) {
        if let Self::Variable(id) = self {
            if let Some(domain_graph_node) = domain_graph_node {
                let domain = domain_graph.domain(&domain_graph_node);
                domain_map
                    .entry(id.clone())
                    .and_modify(|existing| existing.retain(|element| domain.contains(element)))
                    .or_insert(domain);
            }
        }
    }
}

impl<Id: Clone + Ord> Rule<Id> {
    pub fn domain_map(
        &self,
        distinct: &Id,
        or: &Id,
        domain_graph: &DomainGraph<Id>,
    ) -> DomainMap<Id> {
        let mut domain_map = BTreeMap::new();
        self.term
            .domain_map(distinct, or, &mut domain_map, domain_graph, None);
        self.predicates.iter().for_each(|predicate| {
            predicate.domain_map(distinct, or, &mut domain_map, domain_graph);
        });
        domain_map
    }
}

impl<Id: Clone + Ord> Predicate<Id> {
    fn domain_map(
        &self,
        distinct: &Id,
        or: &Id,
        domain_map: &mut DomainMap<Id>,
        domain_graph: &DomainGraph<Id>,
    ) {
        self.term
            .domain_map(distinct, or, domain_map, domain_graph, None);
    }
}

impl<Id: Clone + Ord> Term<Id> {
    fn domain_map(
        &self,
        distinct: &Id,
        or: &Id,
        domain_map: &mut DomainMap<Id>,
        domain_graph: &DomainGraph<Id>,
        domain_graph_node: Option<DomainGraphNode<Id>>,
    ) {
        match self {
            Self::Base(proposition) => proposition.domain_map(
                distinct,
                or,
                domain_map,
                domain_graph,
                Some(DomainGraphNode::Order(Self::ORDER_BASE, 0)),
            ),
            Self::Custom0(name) => name.domain_map(domain_map, domain_graph, domain_graph_node),
            Self::CustomN(AtomOrVariable::Atom(id), arguments) => {
                for (index, argument) in arguments.iter().enumerate() {
                    argument.domain_map(
                        distinct,
                        or,
                        domain_map,
                        domain_graph,
                        if id == distinct || id == or {
                            None
                        } else {
                            Some(DomainGraphNode::Custom(id.clone(), index))
                        },
                    );
                }
            }
            Self::CustomN(AtomOrVariable::Variable(_), _) => unreachable!(),
            Self::Does(role, action) => {
                role.domain_map(
                    domain_map,
                    domain_graph,
                    Some(DomainGraphNode::Order(Self::ORDER_DOES, 0)),
                );
                action.domain_map(
                    distinct,
                    or,
                    domain_map,
                    domain_graph,
                    Some(DomainGraphNode::Order(Self::ORDER_DOES, 1)),
                );
            }
            Self::Goal(role, utility) => {
                role.domain_map(
                    domain_map,
                    domain_graph,
                    Some(DomainGraphNode::Order(Self::ORDER_DOES, 0)),
                );
                utility.domain_map(
                    domain_map,
                    domain_graph,
                    Some(DomainGraphNode::Order(Self::ORDER_DOES, 1)),
                );
            }
            Self::Init(proposition) => proposition.domain_map(
                distinct,
                or,
                domain_map,
                domain_graph,
                Some(DomainGraphNode::Order(Self::ORDER_INIT, 0)),
            ),
            Self::Input(role, action) => {
                role.domain_map(
                    domain_map,
                    domain_graph,
                    Some(DomainGraphNode::Order(Self::ORDER_INPUT, 0)),
                );
                action.domain_map(
                    distinct,
                    or,
                    domain_map,
                    domain_graph,
                    Some(DomainGraphNode::Order(Self::ORDER_INPUT, 1)),
                );
            }
            Self::Legal(role, action) => {
                role.domain_map(
                    domain_map,
                    domain_graph,
                    Some(DomainGraphNode::Order(Self::ORDER_LEGAL, 0)),
                );
                action.domain_map(
                    distinct,
                    or,
                    domain_map,
                    domain_graph,
                    Some(DomainGraphNode::Order(Self::ORDER_LEGAL, 1)),
                );
            }
            Self::Next(proposition) => proposition.domain_map(
                distinct,
                or,
                domain_map,
                domain_graph,
                Some(DomainGraphNode::Order(Self::ORDER_NEXT, 0)),
            ),
            Self::Role(role) => {
                role.domain_map(
                    domain_map,
                    domain_graph,
                    Some(DomainGraphNode::Order(Self::ORDER_ROLE, 0)),
                );
            }
            Self::Terminal => {}
            Self::True(proposition) => proposition.domain_map(
                distinct,
                or,
                domain_map,
                domain_graph,
                Some(DomainGraphNode::Order(Self::ORDER_TRUE, 0)),
            ),
        }
    }
}
