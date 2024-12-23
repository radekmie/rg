use crate::ast::{AtomOrVariable, Game, Term};
use std::collections::BTreeSet;
use std::fmt::Display;

impl<Id: Clone> AtomOrVariable<Id> {
    fn as_indexed_variable(&self, index: usize) -> Option<(usize, Id)> {
        match self {
            Self::Atom(_) => None,
            Self::Variable(id) => Some((index, id.clone())),
        }
    }

    fn to_domain_graph_node(&self) -> Option<DomainGraphNode<Id>> {
        match self {
            Self::Atom(id) => Some(DomainGraphNode::Constant(id.clone())),
            Self::Variable(_) => None,
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct DomainGraph<Id> {
    edges: BTreeSet<(DomainGraphNode<Id>, DomainGraphNode<Id>)>,
}

impl<Id: Ord> Default for DomainGraph<Id> {
    fn default() -> Self {
        use DomainGraphNode::Order;
        let edges = BTreeSet::from([
            (
                Order(Term::<()>::ORDER_BASE, 0),
                Order(Term::<()>::ORDER_TRUE, 0),
            ),
            (
                Order(Term::<()>::ORDER_INPUT, 0),
                Order(Term::<()>::ORDER_DOES, 0),
            ),
            (
                Order(Term::<()>::ORDER_INPUT, 1),
                Order(Term::<()>::ORDER_DOES, 1),
            ),
        ]);

        Self { edges }
    }
}

impl<Id: Display> DomainGraph<Id> {
    pub fn to_graphviz(&self) -> String {
        let mut graphviz = String::new();
        graphviz.push_str("digraph {\n");
        for (lhs, rhs) in &self.edges {
            graphviz.push_str("  ");
            lhs.to_graphviz(&mut graphviz);
            graphviz.push_str(" -> ");
            rhs.to_graphviz(&mut graphviz);
            graphviz.push('\n');
        }
        graphviz.push('}');
        graphviz
    }
}

impl<Id: Clone + Ord> DomainGraph<Id> {
    pub fn domain(&self, node: &DomainGraphNode<Id>) -> BTreeSet<Id> {
        fn step<'a, Id: Clone + Ord>(
            domain_graph: &'a DomainGraph<Id>,
            node: &'a DomainGraphNode<Id>,
            domain: &mut BTreeSet<Id>,
            seen: &mut BTreeSet<&'a DomainGraphNode<Id>>,
        ) {
            if let DomainGraphNode::Constant(id) = node {
                domain.insert(id.clone());
            } else {
                for (lhs, rhs) in &domain_graph.edges {
                    if rhs == node && seen.insert(lhs) {
                        step(domain_graph, lhs, domain, seen);
                    }
                }
            }
        }

        let mut domain = BTreeSet::new();
        let mut seen = BTreeSet::from([node]);
        step(self, node, &mut domain, &mut seen);
        domain
    }
}

impl<Id: Ord> DomainGraph<Id> {
    fn insert(&mut self, lhs: DomainGraphNode<Id>, rhs: DomainGraphNode<Id>) {
        self.edges.insert((lhs, rhs));
    }
}

#[derive(Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum DomainGraphNode<Id> {
    Constant(Id),
    Custom(Id, usize),
    Order(u8, usize),
}

impl<Id: Display> DomainGraphNode<Id> {
    fn to_graphviz(&self, graphviz: &mut String) {
        if let Self::Custom(_, _) | Self::Order(_, _) = &self {
            graphviz.push('"');
        }

        match &self {
            Self::Constant(id) | Self::Custom(id, _) => graphviz.push_str(&id.to_string()),
            Self::Order(Term::<()>::ORDER_BASE, _) => graphviz.push_str("base"),
            Self::Order(Term::<()>::ORDER_DOES, _) => graphviz.push_str("does"),
            Self::Order(Term::<()>::ORDER_GOAL, _) => graphviz.push_str("goal"),
            Self::Order(Term::<()>::ORDER_INIT, _) => graphviz.push_str("init"),
            Self::Order(Term::<()>::ORDER_INPUT, _) => graphviz.push_str("input"),
            Self::Order(Term::<()>::ORDER_LEGAL, _) => graphviz.push_str("legal"),
            Self::Order(Term::<()>::ORDER_NEXT, _) => graphviz.push_str("next"),
            Self::Order(Term::<()>::ORDER_ROLE, _) => graphviz.push_str("role"),
            Self::Order(Term::<()>::ORDER_TERMINAL, _) => graphviz.push_str("terminal"),
            Self::Order(Term::<()>::ORDER_TRUE, _) => graphviz.push_str("true"),
            _ => unreachable!(),
        }

        if let Self::Custom(_, index) | Self::Order(_, index) = &self {
            graphviz.push('[');
            graphviz.push_str(&index.to_string());
            graphviz.push_str("]\"");
        }
    }
}

impl<Id: Clone + Ord> Game<Id> {
    /// Definition 14.1 of <http://logic.stanford.edu/ggp/notes/chapter_14.html>.
    pub fn domain_graph(&self) -> DomainGraph<Id> {
        // 5.
        let mut graph = DomainGraph::default();

        // 3.
        for term in self.subterms() {
            match term {
                Term::Base(proposition)
                | Term::Init(proposition)
                | Term::Next(proposition)
                | Term::True(proposition) => {
                    if proposition.as_atom().is_some() {
                        graph.insert(
                            proposition.to_domain_graph_node(None),
                            term.to_domain_graph_node(Some(0)),
                        );
                    }
                }
                Term::Custom0(_) => {}
                Term::CustomN(name, arguments) => {
                    if name.is_atom() {
                        for (index, argument) in arguments.iter().enumerate() {
                            if argument.as_atom().is_some() {
                                graph.insert(
                                    argument.to_domain_graph_node(None),
                                    term.to_domain_graph_node(Some(index)),
                                );
                            }
                        }
                    }
                }
                Term::Does(role, action)
                | Term::Input(role, action)
                | Term::Legal(role, action) => {
                    if let Some(role) = role.to_domain_graph_node() {
                        graph.insert(role, term.to_domain_graph_node(Some(0)));
                    }
                    if action.as_atom().is_some() {
                        graph.insert(
                            action.to_domain_graph_node(None),
                            term.to_domain_graph_node(Some(1)),
                        );
                    }
                }
                Term::Goal(role, utility) => {
                    if let Some(role) = role.to_domain_graph_node() {
                        graph.insert(role, term.to_domain_graph_node(Some(0)));
                    }
                    if let Some(utility) = utility.to_domain_graph_node() {
                        graph.insert(utility, term.to_domain_graph_node(Some(1)));
                    }
                }
                Term::Role(role) => {
                    if let Some(role) = role.to_domain_graph_node() {
                        graph.insert(role, term.to_domain_graph_node(Some(0)));
                    }
                }
                Term::Terminal => {}
            }
        }

        // 4.
        for rule in &self.0 {
            let head_terms: Vec<_> = rule
                .term
                .subterms()
                .filter(|term| term.has_variable())
                .cloned()
                .collect();
            if head_terms.is_empty() {
                continue;
            }

            for predicate in &rule.predicates {
                for body_term in predicate.term.subterms().filter(|term| term.has_variable()) {
                    for body_var in body_term.variables() {
                        for head_term in &head_terms {
                            for head_var in head_term.variables() {
                                if body_var.1 == head_var.1 {
                                    graph.insert(
                                        body_term.to_domain_graph_node(Some(body_var.0)),
                                        head_term.to_domain_graph_node(Some(head_var.0)),
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }

        graph
    }
}

impl<Id: Clone> Term<Id> {
    fn as_indexed_variable(&self, index: usize) -> Option<(usize, Id)> {
        match self {
            Self::Custom0(atom_or_variable) => atom_or_variable.as_indexed_variable(index),
            _ => None,
        }
    }

    fn to_domain_graph_node(&self, index: Option<usize>) -> DomainGraphNode<Id> {
        use DomainGraphNode::{Constant, Custom, Order};
        match self {
            Self::Base(_) => Order(Self::ORDER_BASE, index.unwrap()),
            Self::Custom0(AtomOrVariable::Atom(id))
            | Self::CustomN(AtomOrVariable::Atom(id), _) => {
                index.map_or_else(|| Constant(id.clone()), |index| Custom(id.clone(), index))
            }
            Self::Custom0(_) => todo!("Custom0"),
            Self::CustomN(_, _) => todo!("CustomN"),
            Self::Does(_, _) => Order(Self::ORDER_DOES, index.unwrap()),
            Self::Goal(_, _) => Order(Self::ORDER_GOAL, index.unwrap()),
            Self::Init(_) => Order(Self::ORDER_INIT, index.unwrap()),
            Self::Input(_, _) => Order(Self::ORDER_INPUT, index.unwrap()),
            Self::Legal(_, _) => Order(Self::ORDER_LEGAL, index.unwrap()),
            Self::Next(_) => Order(Self::ORDER_NEXT, index.unwrap()),
            Self::Role(_) => Order(Self::ORDER_ROLE, index.unwrap()),
            Self::Terminal => todo!("Terminal"),
            Self::True(_) => Order(Self::ORDER_TRUE, index.unwrap()),
        }
    }

    fn variables(&self) -> Vec<(usize, Id)> {
        fn merge<Id>(v1: Option<(usize, Id)>, v2: Option<(usize, Id)>) -> Vec<(usize, Id)> {
            match (v1, v2) {
                (None, None) => vec![],
                (Some(v), None) | (None, Some(v)) => vec![v],
                (Some(v1), Some(v2)) => vec![v1, v2],
            }
        }

        match self {
            Self::Base(proposition) => merge(proposition.as_indexed_variable(0), None),
            Self::Custom0(_) => todo!("Custom0"),
            Self::CustomN(AtomOrVariable::Atom(_), arguments) => arguments
                .iter()
                .enumerate()
                .filter_map(|(index, argument)| argument.as_indexed_variable(index))
                .collect(),
            Self::CustomN(_, _) => todo!("CustomN"),
            Self::Does(role, action) => {
                merge(role.as_indexed_variable(0), action.as_indexed_variable(1))
            }
            Self::Goal(role, utility) => {
                merge(role.as_indexed_variable(0), utility.as_indexed_variable(1))
            }
            Self::Init(_) => todo!("Init"),
            Self::Input(role, action) => {
                merge(role.as_indexed_variable(0), action.as_indexed_variable(1))
            }
            Self::Legal(role, action) => {
                merge(role.as_indexed_variable(0), action.as_indexed_variable(1))
            }
            Self::Next(proposition) => merge(proposition.as_indexed_variable(0), None),
            Self::Role(role) => merge(role.as_indexed_variable(0), None),
            Self::Terminal => todo!("Terminal"),
            Self::True(proposition) => merge(proposition.as_indexed_variable(0), None),
        }
    }
}

#[cfg(test)]
mod test {
    use super::{DomainGraph, DomainGraphNode};
    use crate::ast::{Game, Term};
    use crate::parser::game;
    use nom::combinator::all_consuming;
    use std::collections::BTreeSet;

    fn parse(input: &str) -> Game<&str> {
        all_consuming(game)(input).unwrap().1
    }

    #[test]
    fn example_1() {
        use DomainGraphNode::{Constant, Custom, Order};
        let game = parse("base(cell(M, N, x)) :- index(M) & index(N)");
        let actual = game.domain_graph();

        let mut expect = DomainGraph::default();
        expect.insert(Constant("cell"), Order(Term::<()>::ORDER_BASE, 0));
        expect.insert(Constant("x"), Custom("cell", 2));
        expect.insert(Custom("index", 0), Custom("cell", 0));
        expect.insert(Custom("index", 0), Custom("cell", 1));
        assert_eq!(actual.to_graphviz(), expect.to_graphviz());
    }

    #[test]
    fn example_2() {
        use DomainGraphNode::{Constant, Custom, Order};
        let game = parse(
            "
            line(X) :- row(M, X)
            line(X) :- column(N, X)
            line(X) :- diagonal(X)

            row(M, X) :-
              true(cell(M, 1, X)) &
              true(cell(M, 2, X)) &
              true(cell(M, 3, X))

            column(N, X) :-
              true(cell(1, N, X)) &
              true(cell(2, N, X)) &
              true(cell(3, N, X))

            diagonal(X) :-
              true(cell(1, 1, X)) &
              true(cell(2, 2, X)) &
              true(cell(3, 3, X))

            diagonal(X) :-
              true(cell(1, 3, X)) &
              true(cell(2, 2, X)) &
              true(cell(3, 1, X))

            index(1)
            index(2)
            index(3)

            base(cell(M, N, x)) :- index(M) & index(N)
            base(cell(M, N, o)) :- index(M) & index(N)
            base(cell(M, N, b)) :- index(M) & index(N)
            ",
        );
        let actual = game.domain_graph();

        let mut expect = DomainGraph::default();
        expect.insert(Constant("1"), Custom("cell", 0));
        expect.insert(Constant("1"), Custom("cell", 1));
        expect.insert(Constant("1"), Custom("index", 0));
        expect.insert(Constant("2"), Custom("cell", 0));
        expect.insert(Constant("2"), Custom("cell", 1));
        expect.insert(Constant("2"), Custom("index", 0));
        expect.insert(Constant("3"), Custom("cell", 0));
        expect.insert(Constant("3"), Custom("cell", 1));
        expect.insert(Constant("3"), Custom("index", 0));
        expect.insert(Constant("b"), Custom("cell", 2));
        expect.insert(Constant("cell"), Order(Term::<()>::ORDER_BASE, 0));
        expect.insert(Constant("cell"), Order(Term::<()>::ORDER_TRUE, 0));
        expect.insert(Constant("o"), Custom("cell", 2));
        expect.insert(Constant("x"), Custom("cell", 2));
        expect.insert(Custom("cell", 0), Custom("row", 0));
        expect.insert(Custom("cell", 1), Custom("column", 0));
        expect.insert(Custom("cell", 2), Custom("column", 1));
        expect.insert(Custom("cell", 2), Custom("diagonal", 0));
        expect.insert(Custom("cell", 2), Custom("row", 1));
        expect.insert(Custom("column", 1), Custom("line", 0));
        expect.insert(Custom("diagonal", 0), Custom("line", 0));
        expect.insert(Custom("index", 0), Custom("cell", 0));
        expect.insert(Custom("index", 0), Custom("cell", 1));
        expect.insert(Custom("row", 1), Custom("line", 0));
        assert_eq!(actual.to_graphviz(), expect.to_graphviz());

        let domain_1 = BTreeSet::from(["1", "2", "3"]);
        let domain_2 = BTreeSet::from(["b", "o", "x"]);
        assert_eq!(actual.domain(&Custom("cell", 0)), domain_1);
        assert_eq!(actual.domain(&Custom("cell", 1)), domain_1);
        assert_eq!(actual.domain(&Custom("cell", 2)), domain_2);
        assert_eq!(actual.domain(&Custom("column", 0)), domain_1);
        assert_eq!(actual.domain(&Custom("column", 1)), domain_2);
        assert_eq!(actual.domain(&Custom("diagonal", 0)), domain_2);
        assert_eq!(actual.domain(&Custom("line", 0)), domain_2);
        assert_eq!(actual.domain(&Custom("row", 0)), domain_1);
        assert_eq!(actual.domain(&Custom("row", 1)), domain_2);
    }

    #[test]
    fn example_4() {
        use DomainGraphNode::{Custom, Order};
        let game = parse(include_str!("../../../../games/kif/ticTacToe.kif"));
        let graph = game.domain_graph();
        println!("{}", graph.to_graphviz());

        let domain_1 = BTreeSet::from(["1", "2", "3"]);
        let domain_2 = BTreeSet::from(["b", "o", "x"]);
        let domain_3 = BTreeSet::from(["oplayer", "xplayer"]);
        let domain_4 = BTreeSet::from(["cell", "control"]);
        let domain_5 = BTreeSet::from(["mark", "noop"]);
        let domain_6 = BTreeSet::from(["0", "50", "100"]);
        assert_eq!(graph.domain(&Custom("cell", 0)), domain_1);
        assert_eq!(graph.domain(&Custom("cell", 1)), domain_1);
        assert_eq!(graph.domain(&Custom("cell", 2)), domain_2);
        assert_eq!(graph.domain(&Custom("control", 0)), domain_3);
        assert_eq!(graph.domain(&Custom("index", 0)), domain_1);
        assert_eq!(graph.domain(&Custom("mark", 0)), domain_1);
        assert_eq!(graph.domain(&Custom("mark", 1)), domain_1);
        assert_eq!(graph.domain(&Order(Term::<()>::ORDER_BASE, 0)), domain_4);
        assert_eq!(graph.domain(&Order(Term::<()>::ORDER_DOES, 0)), domain_3);
        assert_eq!(graph.domain(&Order(Term::<()>::ORDER_DOES, 1)), domain_5);
        assert_eq!(graph.domain(&Order(Term::<()>::ORDER_GOAL, 0)), domain_3);
        assert_eq!(graph.domain(&Order(Term::<()>::ORDER_GOAL, 1)), domain_6);
        assert_eq!(graph.domain(&Order(Term::<()>::ORDER_INIT, 0)), domain_4);
        assert_eq!(graph.domain(&Order(Term::<()>::ORDER_INPUT, 0)), domain_3);
        assert_eq!(graph.domain(&Order(Term::<()>::ORDER_INPUT, 1)), domain_5);
        assert_eq!(graph.domain(&Order(Term::<()>::ORDER_LEGAL, 0)), domain_3);
        assert_eq!(graph.domain(&Order(Term::<()>::ORDER_LEGAL, 1)), domain_5);
        assert_eq!(graph.domain(&Order(Term::<()>::ORDER_NEXT, 0)), domain_4);
        assert_eq!(graph.domain(&Order(Term::<()>::ORDER_ROLE, 0)), domain_3);
        assert_eq!(graph.domain(&Order(Term::<()>::ORDER_TRUE, 0)), domain_4);
    }
}
