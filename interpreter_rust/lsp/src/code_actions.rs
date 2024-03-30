use crate::document::AST;

use super::common::utils::ToLspRange;
use rg::ast::Game;
use std::collections::HashMap;
use tower_lsp::lsp_types::{
    CodeAction, CodeActionKind, CodeActionOrCommand, CodeActionResponse, TextEdit, Url,
    WorkspaceEdit,
};
use utils::{
    position::{Positioned, Span},
    Identifier,
};

pub fn provide(uri: &Url, span: &Span, game: &AST, text: &str) -> Option<CodeActionResponse> {
    match game {
        AST::RG(game) => {
            let actions = vec![split_edge(uri, span, game, text)]
                .into_iter()
                .flatten()
                .collect();
            Some(actions)
        }
        AST::HRG(_) => None,
    }
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
    let left_split = edge.rhs.span().start > span.end;
    let comma_pos = line.find(',')?;
    let (lhs, rhs_with_label) = line.split_at(comma_pos);
    let new_text = if left_split {
        format!("{lhs}, new_edge: ;\nnew_edge{rhs_with_label}")
    } else {
        let last_colon_pos = rhs_with_label.rfind(':')?;
        let (rhs, label) = rhs_with_label.split_at(last_colon_pos);
        format!("{lhs}, new_edge{label}\nnew_edge{rhs}: ;")
    };
    let text_edits = vec![TextEdit {
        range: edge.span().to_lsp(),
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
