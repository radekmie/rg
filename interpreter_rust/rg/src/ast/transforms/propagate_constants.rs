// use crate::ast::analyses::ConstantsAnalysis;
// use crate::ast::{Edge, Error, Expression, Game, Label, Node, Value};
// use std::ops::DerefMut;
// use std::sync::Arc;
// use std::collections::BTreeMap;

// type Id = Arc<str>;
// type Analysis = BTreeMap<Id, Arc<Value<Id>>>;

// impl Game<Id> {
//     pub fn propagate_constants(&mut self) -> Result<(), Error<Id>> {
//         let analysis = self.analyse::<ConstantsAnalysis>(true);

//         for edge in &mut self.edges {
//             replace_in_label(&mut edge.label, &analysis);
//         }

//         Ok(())
//     }
// }

// fn replace_in_label(label: &mut Label<Id>, analysis: &Analysis) {
//     match label {
//         Label::Assignment { lhs, rhs } => todo!(),
//         Label::Comparison { lhs, rhs, .. } => {
//             replace_in_expression(lhs, analysis);
//             replace_in_expression(rhs, analysis);
//         },
//         _ => (),
//     }
// }

// fn replace_in_expression(expression: &mut Expression<Id>, analysis: &Analysis) {
//     match expression {
//         Expression::Access { span, lhs, rhs } => todo!(),
//         Expression::Cast { rhs, .. } => replace_in_expression(rhs.deref_mut(), analysis),
//         Expression::Reference { identifier } => {
//             // PROBLEM: Value can be a map
//             if let Some(value) = analysis.get(identifier) {
//                 *expression = Expression::Reference { identifier: () } { value: value.clone() };
//             }
//         },
//     }   
// }