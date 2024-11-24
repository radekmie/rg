use clap::{Args, Parser};
use interpreter::{
    analyze_gdl_inner as gdl, analyze_hrg_inner as hrg, analyze_rg_inner as rg, prepare_ist, Flags,
};
use rand::thread_rng;
use rg::ast::Game;
use std::ffi::OsStr;
use std::fs::read_to_string;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Parser)]
#[command(about, version)]
enum CliArgs {
    Perf {
        #[command(flatten)]
        game_with_flags: GameWithFlags,
        depth: usize,
    },
    Run {
        #[command(flatten)]
        game_with_flags: GameWithFlags,
        plays: usize,
    },
    Source {
        #[command(flatten)]
        game_with_flags: GameWithFlags,
    },
}

#[derive(Args)]
struct GameWithFlags {
    #[command(flatten)]
    flags: Flags,
    path: PathBuf,
}

impl GameWithFlags {
    fn load(self) -> Result<Game<Arc<str>>, String> {
        let Self { flags, path } = self;
        let callback = None::<fn(_)>;
        let source = read_to_string(&path).map_err(|error| error.to_string())?;
        let source_rg = match path.extension().and_then(OsStr::to_str) {
            Some("hrg") => hrg(&source, flags.reuse_functions, callback)?.to_string(),
            Some("kif") => gdl(&source, callback)?.to_string(),
            Some("rg") => source,
            _ => panic!("Unknown game type: {}.", path.display()),
        };

        rg(&source_rg, &flags, callback)
    }
}

fn main() -> Result<(), String> {
    match CliArgs::parse() {
        CliArgs::Perf {
            depth,
            game_with_flags,
        } => {
            let game = prepare_ist(game_with_flags.load()?)?.0;
            for depth in 0..=depth {
                let (count, time) = game.perf(depth);
                println!("perf(depth: {depth}) = {count} in {time:.3}ms",);
            }
        }
        CliArgs::Run {
            game_with_flags,
            plays,
        } => {
            let (game, interner) = prepare_ist(game_with_flags.load()?)?;
            game.run(
                &mut thread_rng(),
                &interner,
                plays,
                &Some(|lines: Vec<_>| {
                    println!("{esc}c{}", lines.join("\n"), esc = 27 as char);
                }),
            );
        }
        CliArgs::Source { game_with_flags } => {
            let game = game_with_flags.load()?;
            println!("{game}");
        }
    }

    Ok(())
}
