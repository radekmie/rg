use crate::ast::{Error, Game, Pragma};
use std::collections::BTreeSet;
use std::sync::Arc;

impl<Id: Clone + Ord> Game<Id> {
    pub fn skip_artificial_tags(&mut self) -> Result<(), Error<Id>> {
        let mut artificial_tags = BTreeSet::new();
        self.pragmas.retain(|pragma| {
            if let Pragma::ArtificialTag { tags, .. } = pragma {
                for tag in tags {
                    artificial_tags.insert(tag.clone());
                }
                return false;
            }

            true
        });

        for edge in &mut self.edges {
            if edge.label.is_tag_and(|tag| artificial_tags.contains(tag)) {
                Arc::make_mut(edge).skip();
            }
        }

        Ok(())
    }
}
