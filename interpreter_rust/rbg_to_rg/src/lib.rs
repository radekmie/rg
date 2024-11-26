use rbg::ast as rbg;
use rg::ast as rg;
use std::sync::Arc;

type Id = Arc<str>;

pub fn rbg_to_rg(_rbg: &rbg::Game<Id>) -> Result<rg::Game<Id>, rbg::Error<Id>> {
    todo!("RBG to RG translation")
}
