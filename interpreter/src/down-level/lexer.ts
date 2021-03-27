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
export const KeywordDomain = token('KeywordDomain', /domain/, Identifier);
export const KeywordIn = token('KeywordIn', /in/, Identifier);
export const KeywordOr = token('KeywordOr', /or/, Identifier);
export const KeywordWhere = token('KeywordWhere', /where/, Identifier);

// Symbols.
export const BraceLeft = token('BraceLeft', /{/);
export const BraceRight = token('BraceRight', /}/);
export const BracketLeft = token('BracketLeft', /\[/);
export const BracketRight = token('BracketRight', /]/);
export const Colon = token('Colon', /:/);
export const Comma = token('Comma', /,/);
export const Dash = token('Dash', /-/);
export const DashGt = token('Arrow', /->/);
export const DotDot = token('DotDot', /\.\./);
export const Equal = token('Equal', /=/);
export const EqualEqual = token('EqualEqual', /==/);
export const Gt = token('Gt', />/);
export const GtEqual = token('GtEqual', />=/);
export const Lt = token('Lt', /</);
export const LtEqual = token('LtEqual', /<=/);
export const Not = token('Not', /!/);
export const NotEqual = token('NotEqual', /!=/);
export const Or = token('Or', /\|/);
export const OrOr = token('OrOr', /\|\|/);
export const ParenthesisLeft = token('ParenthesisLeft', /\(/);
export const ParenthesisRight = token('ParenthesisRight', /\)/);

// Removables.
export const Comment = token('Comment', /\/\/.*?(\n|\r\n?)/, null);
export const WhiteSpace = token('WhiteSpace', /\s+/, null);

// Create shared lexer instance.
export default new Lexer(tokens);
