import { ITokenConfig, Lexer, TokenType, createToken } from 'chevrotain';

export const tokens: TokenType[] = [];

function token(name: string, pattern: RegExp, extra?: TokenType | null) {
  const config: ITokenConfig = { name, pattern };
  if (extra === null) config.group = Lexer.SKIPPED;
  // eslint-disable-next-line @typescript-eslint/camelcase
  if (extra !== null && extra !== undefined) config.longer_alt = extra;

  const token = createToken(config);
  tokens.unshift(token);
  return token;
}

// Generics.
export const Identifier = token('Identifier', /[_a-zA-Z0-9]+/);

// Keywords.
export const KeywordConst = token('KeywordConst', /const/, Identifier);
export const KeywordType = token('KeywordType', /type/, Identifier);
export const KeywordVar = token('KeywordVar', /var/, Identifier);

// Symbols.
export const Arrow = token('Arrow', /->/);
export const Bang = token('Bang', /!/);
export const BangEqual = token('BangEqual', /!=/);
export const BraceLeft = token('BraceLeft', /{/);
export const BraceRight = token('BraceRight', /}/);
export const BracketLeft = token('BracketLeft', /\[/);
export const BracketRight = token('BracketRight', /]/);
export const Colon = token('Colon', /:/);
export const Comma = token('Comma', /,/);
export const Equal = token('Equal', /=/);
export const EqualEqual = token('EqualEqual', /==/);
export const Hash = token('Hash', /#/);
export const ParenthesisLeft = token('ParenthesisLeft', /\(/);
export const ParenthesisRight = token('ParenthesisRight', /\)/);
export const Question = token('Question', /\?/);
export const Semicolon = token('Semicolon', /;/);

// Removables.
export const Comment = token('Comment', /\/\/.*?(\n|\r\n?)/, null);
export const WhiteSpace = token('WhiteSpace', /\s+/, null);

// Create shared lexer instance.
export default new Lexer(tokens);
