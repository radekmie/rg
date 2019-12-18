{
  open Grammar

  exception Error of Lexing.position
}

let number = '-'? (['0'-'9'] | ['1'-'9']['0'-'9']+)
let string = ['a'-'z' 'A'-'Z' '0'-'9' '\'' '_']+

rule token = parse
  (* Whitespaces *)
  | " "    {                         token lexbuf }
  | "\r\n" { Lexing.new_line lexbuf; token lexbuf }
  | "\n"   { Lexing.new_line lexbuf; token lexbuf }
  | "\r"   { Lexing.new_line lexbuf; token lexbuf }
  | "\t"   {                         token lexbuf }

  (* Simples *)
  | "!"  { EXCLAMATION }
  | "("  { PAREN_LEFT }
  | ")"  { PAREN_RIGHT }
  | ","  { COMMA }
  | "->" { ARROW }
  | ":"  { COLON }
  | ";"  { SEMICOLON }
  | "="  { EQUALS }
  | "{"  { CURLY_LEFT }
  | "}"  { CURLY_RIGHT }

  (* Sections *)
  | "#constants" { SECTION_CONSTANTS }
  | "#domains"   { SECTION_DOMAINS }
  | "#init"      { SECTION_INIT }
  | "#rules"     { SECTION_RULES }
  | "#variables" { SECTION_VARIABLES }
  | "#views"     { SECTION_VIEWS }

  (* Comments *)
  | "//" [^'\n']* { token lexbuf }

  (* Values *)
  | number as number { NUMBER(int_of_string number) }
  | string as string { STRING(string) }

  (* Error *)
  | _ { raise @@ Error(Lexing.lexeme_start_p lexbuf) }
