import { ITokenConfig, Lexer, TokenType, createToken } from 'chevrotain';

export const tokens: TokenType[] = [];

function token(name: string, pattern: RegExp, extra?: TokenType | null) {
  const config: ITokenConfig = { name, pattern };
  if (extra === null) {
    config.group = Lexer.SKIPPED;
  }

  if (extra !== null && extra !== undefined) {
    config.longer_alt = extra;
  }

  const token = createToken(config);
  tokens.unshift(token);
  return token;
}

// Notation and ordering follows the original definitions from the extended
// version (https://arxiv.org/abs/1706.02462) of "Regular Boardgames" by
// Jakub Kowalski, Maksymilian Mika, Jakub Sutowicz, and Marek Szykuła.

// Generics.
export const Ident = token('Ident', /[a-zA-Z][a-zA-Z0-9]*/);
export const Nat = token('Nat', /[0-9]+/);

// Symbols.
export const ParenthesisLeft = token('ParenthesisLeft', /\(/);
export const ParenthesisRight = token('ParenthesisRight', /\)/);
export const BraceLeft = token('BraceLeft', /{/);
export const BraceLeftQuestion = token('BraceLeftQuestion', /{\?/);
export const BraceLeftBang = token('BraceLeftBang', /{!/);
export const BraceLeftDollar = token('BraceLeftDollar', /{\$/);
export const BraceRight = token('BraceRight', /}/);
export const BracketLeft = token('BracketLeft', /\[/);
export const BracketLeftDollar = token('BracketLeftDollar', /\[\$/);
export const BracketRight = token('BracketRight', /]/);
export const Hash = token('Hash', /#/);
export const Dash = token('Dash', /-/);
export const Plus = token('Plus', /\+/);
export const Caret = token('Caret', /\^/);
export const Slash = token('Slash', /\//);
export const Star = token('Star', /\*/);
export const Comma = token('Comma', /,/);
export const Semicolon = token('Semicolon', /;/);
export const Colon = token('Colon', /:/);
export const Dollar = token('Dollar', /\$/);
export const Equal = token('Equal', /=/);
export const DashGt = token('DashGt', /->/);
export const DashGtGt = token('DashGtGt', /->>/);
export const Bang = token('Bang', /!/);
export const Question = token('Question', /\?/);
export const BangEqual = token('BangEqual', /!=/);
export const EqualEqual = token('EqualEqual', /==/);
export const Lt = token('Lt', /</);
export const LtEqual = token('LtEqual', /<=/);
export const Gt = token('Gt', />/);
export const GtEqual = token('GtEqual', />=/);

// Keywords.
export const KeywordPlayers = token('KeywordPlayers', /players/, Ident);
export const KeywordPieces = token('KeywordPieces', /pieces/, Ident);
export const KeywordVariables = token('KeywordVariables', /variables/, Ident);
export const KeywordRules = token('KeywordRules', /rules/, Ident);
export const KeywordBoard = token('KeywordBoard', /board/, Ident);

// Removables.
export const CommentSingle = token('CommentSingle', /\/\/.*?(\n|\r\n?)/, null);
export const CommentMulti = token('CommentMulti', /\/\*.*?\*\//, null);
export const WhiteSpace = token('WhiteSpace', /\s+/, null);
