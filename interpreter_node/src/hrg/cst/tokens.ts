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

// Generics.
export const Identifier = token('Identifier', /[_a-zA-Z0-9]+/);

// Keywords.
export const KeywordBranch = token('KeywordBranch', /branch/, Identifier);
export const KeywordDomain = token('KeywordDomain', /domain/, Identifier);
export const KeywordElse = token('KeywordElse', /else/, Identifier);
export const KeywordForall = token('KeywordForall', /forall/, Identifier);
export const KeywordGraph = token('KeywordGraph', /graph/, Identifier);
export const KeywordIf = token('KeywordIf', /if/, Identifier);
export const KeywordIn = token('KeywordIn', /in/, Identifier);
export const KeywordLoop = token('KeywordLoop', /loop/, Identifier);
export const KeywordOr = token('KeywordOr', /or/, Identifier);
export const KeywordThen = token('KeywordThen', /then/, Identifier);
export const KeywordWhen = token('KeywordWhen', /when/, Identifier);
export const KeywordWhere = token('KeywordWhere', /where/, Identifier);
export const KeywordWhile = token('KeywordWhile', /while/, Identifier);
export const KeywordWildcard = token('KeywordWildcard', /_/, Identifier);

// Symbols.
export const AndAnd = token('AndAnd', /&&/);
export const At = token('At', /@/);
export const BraceLeft = token('BraceLeft', /{/);
export const BraceRight = token('BraceRight', /}/);
export const BracketLeft = token('BracketLeft', /\[/);
export const BracketRight = token('BracketRight', /]/);
export const Colon = token('Colon', /:/);
export const Comma = token('Comma', /,/);
export const Dollar = token('Dollar', /\$/);
export const DotDot = token('DotDot', /\.\./);
export const Equal = token('Equal', /=/);
export const EqualEqual = token('EqualEqual', /==/);
export const Gt = token('Gt', />/);
export const GtEqual = token('GtEqual', />=/);
export const Lt = token('Lt', /</);
export const LtEqual = token('LtEqual', /<=/);
export const Minus = token('Minus', /-/);
export const MinusGt = token('MinusGt', /->/);
export const Not = token('Not', /!/);
export const NotEqual = token('NotEqual', /!=/);
export const Or = token('Or', /\|/);
export const OrOr = token('OrOr', /\|\|/);
export const ParenthesisLeft = token('ParenthesisLeft', /\(/);
export const ParenthesisRight = token('ParenthesisRight', /\)/);
export const Plus = token('Plus', /\+/);

// Removables.
export const Comment = token('Comment', /\/\/.*?(\n|\r\n?)/, null);
export const WhiteSpace = token('WhiteSpace', /\s+/, null);
