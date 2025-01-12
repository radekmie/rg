use clap::{Args, Parser};
use interpreter::{analyze_inner, Flags, Game};
use rand::thread_rng;
use rg::ast::Game as GameAst;
use serde_json::{from_str, json, Value};
use std::ffi::OsStr;
use std::fs::read_to_string;
use std::path::PathBuf;
use std::process::exit;
use std::sync::Arc;

#[derive(Parser)]
#[command(about, version)]
/// Regular Games CLI
enum CliArgs {
    /// Print RG AST
    Ast {
        #[command(flatten)]
        game_with_flags: GameWithFlags,
    },
    /// Print formatted source
    Format { path: PathBuf },
    /// Benchmark game tree
    Perf {
        #[command(flatten)]
        game_with_flags: GameWithFlags,
        depth: usize,
    },
    /// Benchmark random playouts
    Run {
        #[command(flatten)]
        game_with_flags: GameWithFlags,
        plays: usize,
    },
    /// Print RG source
    Source {
        #[command(flatten)]
        game_with_flags: GameWithFlags,
    },
    /// Print game stats
    Stats {
        #[command(flatten)]
        game_with_flags: GameWithFlags,
        #[arg(long)]
        json: bool,
    },
}

#[derive(Args)]
struct GameWithFlags {
    #[command(flatten)]
    flags: Flags,
    path: PathBuf,
    /// prints detailed timings of all transformations
    #[arg(long)]
    verbose: bool,
}

impl GameWithFlags {
    fn load(self) -> Result<GameAst<Arc<str>>, String> {
        self.load_with_callback(&mut None::<fn(String)>)
    }

    fn load_with_callback(
        self,
        callback: &mut Option<impl FnMut(String)>,
    ) -> Result<GameAst<Arc<str>>, String> {
        let Self {
            flags,
            path,
            verbose,
        } = self;
        let Some(extension) = path.extension().and_then(OsStr::to_str) else {
            return Err(format!("Unknown game type: {}.", path.display()));
        };

        let source = read_to_string(&path).map_err(|error| error.to_string())?;
        analyze_inner(source, extension, &flags, verbose, callback)
    }

    fn new(path: PathBuf) -> Self {
        let flags = Flags::none();
        Self {
            flags,
            path,
            verbose: false,
        }
    }
}

fn main() -> Result<(), String> {
    match CliArgs::parse() {
        CliArgs::Ast { game_with_flags } => {
            let game = game_with_flags.load()?;
            println!("{}", json!(game));
        }
        CliArgs::Format { path } => {
            let game = GameWithFlags::new(path);

            // TODO: Replace `step` with a struct.
            game.load_with_callback(&mut Some(|step: String| {
                if let Value::Object(mut step) = from_str(&step).unwrap() {
                    if let Some(Value::String(title)) = step.remove("title") {
                        if title.starts_with("formatted") {
                            if let Some(Value::String(source)) = step.remove("value") {
                                println!("{source}");
                                exit(0);
                            }
                        }
                    }
                }
            }))?;

            panic!("No formatted source found");
        }
        CliArgs::Perf {
            depth,
            game_with_flags,
        } => {
            let game = Game::try_from(game_with_flags.load()?)?.0;
            for depth in 0..=depth {
                let (count, time) = game.perf(depth);
                println!("perf(depth: {depth}) = {count} in {time:.3}ms",);
            }
        }
        CliArgs::Run {
            game_with_flags,
            plays,
        } => {
            let (game, interner) = Game::try_from(game_with_flags.load()?)?;
            game.run(
                &mut thread_rng(),
                &interner,
                plays,
                &Some(|lines: Vec<_>| {
                    println!("{esc}c{}", lines.join("\n"), esc = 27 as char);
                }),
            )?;
        }
        CliArgs::Source { game_with_flags } => {
            let game = game_with_flags.load()?;
            println!("{game}");
        }
        CliArgs::Stats { game_with_flags, json } => {
            let game = game_with_flags.load()?;
            let stats = game.to_stats();
            if json {
                println!("{}", json!(stats));
            } else {
                println!("{stats}");
            }
        }
    }

    Ok(())
}
