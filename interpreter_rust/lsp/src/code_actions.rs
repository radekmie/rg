use std::collections::HashMap;

use super::utils::span_to_lsp;
use rg::{
    ast::{Game, Identifier},
    position::{Positioned, Span},
};
use tower_lsp::lsp_types::{
    CodeAction, CodeActionKind, CodeActionOrCommand, CodeActionResponse, TextEdit, Url,
    WorkspaceEdit,
};

pub fn provide(
    uri: &Url,
    span: &Span,
    game: &Game<Identifier>,
    text: &str,
) -> Option<CodeActionResponse> {
    let actions = vec![split_edge(uri, span, game, text)]
        .into_iter()
        .flatten()
        .collect();
    Some(actions)
}

fn split_edge(
    uri: &Url,
    span: &Span,
    game: &Game<Identifier>,
    text: &str,
) -> Option<CodeActionOrCommand> {
    let edge = game
        .edges
        .iter()
        .find(|edge| edge.span().encloses_span(span))?;
    let line = edge.span().line_at(text)?;
    let left_split = edge.rhs.span().start.is_after(&span.end);
    let comma_pos = line.find(',')?;
    let (lhs, rhs_with_label) = line.split_at(comma_pos);
    let new_text = if left_split {
        format!("{}, new_edge: ;\nnew_edge{}", lhs, rhs_with_label)
    } else {
        let last_colon_pos = rhs_with_label.rfind(':')?;
        let (rhs, label) = rhs_with_label.split_at(last_colon_pos);
        format!("{}, new_edge{}\nnew_edge{}: ;", lhs, label, rhs)
    };
    let text_edits = vec![TextEdit {
        range: span_to_lsp(&edge.span()),
        new_text,
    }];
    let changes = HashMap::from([(uri.clone(), text_edits)]);
    Some(CodeActionOrCommand::CodeAction(CodeAction {
        title: "Split edge".to_string(),
        kind: Some(CodeActionKind::REFACTOR),
        edit: Some(WorkspaceEdit {
            changes: Some(changes),
            document_changes: None,
            change_annotations: None,
        }),
        ..CodeAction::default()
    }))
}
