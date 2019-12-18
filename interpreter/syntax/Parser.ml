open Ast

let parse (lexbuf : Lexing.lexbuf) : (Game.t, string) result =
  try
    Ok(Grammar.game Lexer.token lexbuf)
  with
  | Grammar.Error ->
    Error("Syntax error.")
  | Lexer.Error({ pos_bol; pos_cnum; pos_fname; pos_lnum }) ->
    let file = if pos_fname = "" then "(unknown)" else pos_fname in
    let pattern = Printf.sprintf "Unexpected character in %s at %d:%d." in
    let message = pattern file pos_lnum (pos_cnum - pos_bol + 1) in
    Error(message)

let parse_file (filename : string) : (Game.t, string) result =
  let file = open_in filename in
  let lexbuf = Lexing.from_channel file in
  lexbuf.lex_curr_p <- { lexbuf.lex_curr_p with pos_fname = filename };
  let result = parse lexbuf in
  close_in file;
  result

let parse_string (source : string) : (Game.t, string) result =
  let lexbuf = Lexing.from_string source in
  parse lexbuf
