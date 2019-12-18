%token <int> NUMBER
%token <string> STRING
%token ARROW
%token COLON
%token COMMA
%token CURLY_LEFT
%token CURLY_RIGHT
%token EQUALS
%token EXCLAMATION
%token PAREN_LEFT
%token PAREN_RIGHT
%token SECTION_CONSTANTS
%token SECTION_DOMAINS
%token SECTION_INIT
%token SECTION_RULES
%token SECTION_VARIABLES
%token SECTION_VIEWS
%token SEMICOLON

%start <Ast.Game.t> game

%%

assignement:
  | name = id EQUALS value = value
    { (name, value) }

constant:
  | name = id PAREN_LEFT args = values PAREN_RIGHT EQUALS result = result
    { ((name, args), result) }

domain:
  | name = id EQUALS CURLY_LEFT values = values CURLY_RIGHT
    { (name, values) }

game:
  | domains   = section(SECTION_DOMAINS,   separated_list(SEMICOLON, domain))
    constants = section(SECTION_CONSTANTS, separated_list(SEMICOLON, constant))
    variables = section(SECTION_VARIABLES, separated_list(SEMICOLON, variable))
    views     = section(SECTION_VIEWS,     separated_list(SEMICOLON, variable))
    init      = section(SECTION_INIT,      separated_list(SEMICOLON, assignement))
    rules     = section(SECTION_RULES,     list(rule))
    { { constants; domains; init; rules; variables; views; } }

id:
  | s = STRING
    { Ast.Id.Id(s) }

result:
  | EXCLAMATION
    { None }
  | value = value
    { Some(value) }

rule:
  | PAREN_LEFT a = id COMMA b = id PAREN_RIGHT
    { (a, b, Ast.Rule.Empty) }
  | PAREN_LEFT a = id COMMA b = id COMMA ARROW player = id PAREN_RIGHT
    { (a, b, Ast.Rule.Switch(player)) }

section(header, entries):
  | header PAREN_LEFT entries = entries PAREN_RIGHT
    { entries }

value:
  | n = NUMBER
    { Ast.Value.Number(n) }
  | s = STRING
    { Ast.Value.String(s) }

values:
  | values = separated_list(COMMA, value)
    { values }

variable:
  | name = id COLON domain = id
    { (name, domain) }
