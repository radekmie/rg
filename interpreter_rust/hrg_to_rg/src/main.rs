use hrg::parsing::parser;
use hrg_to_rg::hrg_to_rg;
use map_id::MapId;
use std::env::args;
use std::fs::read_to_string;
use std::sync::Arc;

fn main() -> Result<(), String> {
    let args = args().collect::<Vec<_>>();
    let file = args.get(1).expect("game.hrg file expected.");
    let source = read_to_string(file).map_err(|error| error.to_string())?;
    let (hrg, errors) = parser::parse_with_errors(&source);
    if !errors.is_empty() {
        return Err(errors
            .into_iter()
            .map(|error| format!("{error}"))
            .collect::<Vec<_>>()
            .join("\n"));
    }

    let reuse_functions = args.get(2).is_some_and(|arg| arg == "--reuseFunctions");
    let hrg = hrg.map_id(&mut |id| Arc::from(id.identifier.as_str()));
    let rg = hrg_to_rg(hrg, reuse_functions);
    print!("{rg}");
    Ok(())
}
