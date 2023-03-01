use crate::ast::{Error, Game};
use std::collections::BTreeMap;

impl Game<String> {
    pub fn compact_skip_edges(mut self) -> Result<Self, Error<String>> {
        if !self.are_bindings_unique() {
            self.make_bindings_unique();
        }

        while let Some((x, y)) = self.compact_skip_edge_backward() {
            self.edges[x].rhs = self.edges[y].rhs.clone();
            self.edges.remove(y);
        }

        while let Some((x, y)) = self.compact_skip_edge_forward() {
            self.edges[y].lhs = self.edges[x].lhs.clone();
            self.edges.remove(x);
        }

        while let Some(x) = self.compact_skip_edge_single() {
            self.edges.remove(x);
        }

        Ok(self)
    }

    /// Before:
    ///       x       y
    ///   a ----> b ----> c
    ///
    /// After:
    ///       x
    ///   a ----> c
    ///
    /// Conditions:
    ///   1. x != Assignment of `player` OR c has no bindings
    ///   2. y == Skip
    ///   3. b has no other outgoing edges
    ///   4. b has no bindings
    ///   5. there's no other edge between a and c (multiedges are not allowed)
    fn compact_skip_edge_backward(&self) -> Option<(usize, usize)> {
        for (y_index, y) in self.edges.iter().enumerate() {
            if y.label.is_skip()
                && !y.lhs.has_bindings()
                && self.outgoing_edges(&y.lhs).all(|z| z == y)
            {
                for (x_index, x) in self.edges.iter().enumerate() {
                    if x.is_following(y)
                        && (!y.rhs.has_bindings() || x.label.is_player_assignment())
                        && !self.are_connected(&x.lhs, &y.rhs)
                    {
                        return Some((x_index, y_index));
                    }
                }
            }
        }

        None
    }

    /// Before:
    ///       x       y
    ///   a ----> b ----> c
    ///
    /// After:
    ///       y
    ///   a ----> c
    ///
    /// Conditions:
    ///   1. x == Skip
    ///   2. b has no other incoming edges
    ///   3. b has no bindings
    ///   4. there's no other edge between a and c (multiedges are not allowed)
    fn compact_skip_edge_forward(&self) -> Option<(usize, usize)> {
        for (x_index, x) in self.edges.iter().enumerate() {
            if x.label.is_skip()
                && !x.rhs.has_bindings()
                && self.incoming_edges(&x.rhs).all(|z| z == x)
            {
                for (y_index, y) in self.edges.iter().enumerate() {
                    if x.is_following(y) && !self.are_connected(&x.lhs, &y.rhs) {
                        return Some((x_index, y_index));
                    }
                }
            }
        }

        None
    }

    /// Before:
    ///       x
    ///   a ----> b
    ///
    /// After:
    ///
    ///   b
    ///
    /// Conditions:
    ///   1. x == Skip
    ///   2. a has no other incoming edges
    ///   3. a has no other outgoing edges
    ///   4. a has no bindings
    ///   5. a is not `begin`
    fn compact_skip_edge_single(&self) -> Option<usize> {
        for (x_index, x) in self.edges.iter().enumerate() {
            if x.label.is_skip()
                && !x.lhs.has_bindings()
                && !x.lhs.is_begin()
                && self.incoming_edges(&x.lhs).next().is_none()
                && self.outgoing_edges(&x.lhs).all(|y| y == x)
            {
                return Some(x_index);
            }
        }

        None
    }

    fn are_bindings_unique(&self) -> bool {
        let mut binding_to_edge_name = BTreeMap::default();
        for edge in &self.edges {
            for edge_name in [&edge.lhs, &edge.rhs] {
                for (binding, _) in edge_name.bindings() {
                    match binding_to_edge_name.get(binding) {
                        Some(other) => {
                            if other != &edge_name {
                                return false;
                            }
                        }
                        _ => {
                            binding_to_edge_name.insert(binding, edge_name);
                        }
                    }
                }
            }
        }

        true
    }

    fn make_bindings_unique(&mut self) {
        let mut index = 0;

        // Iterate over indexes to eliminate multiple ownership.
        let edges = &mut self.edges;
        for x in 0..edges.len() {
            let mapping = edges[x]
                .rhs
                .bindings()
                .into_iter()
                .map(|(binding, _)| {
                    index += 1;
                    (binding.clone(), format!("bind_{index}"))
                })
                .collect::<BTreeMap<_, _>>();

            if !mapping.is_empty() {
                for y in 0..edges.len() {
                    if x != y {
                        let rebind_lhs =
                            edges[x].is_following(&edges[y]) || edges[x].is_same_lhs(&edges[y]);
                        let rebind_rhs =
                            edges[y].is_following(&edges[x]) || edges[x].is_same_rhs(&edges[y]);

                        if rebind_lhs || rebind_rhs {
                            edges[y].label = edges[y].label.substitute_bindings(&mapping);
                        }

                        if rebind_lhs {
                            edges[y].lhs = edges[y].lhs.substitute_bindings(&mapping);
                        }

                        if rebind_rhs {
                            edges[y].rhs = edges[y].rhs.substitute_bindings(&mapping);
                        }
                    }
                }

                edges[x] = edges[x].substitute_bindings(&mapping);
            }
        }
    }
}
