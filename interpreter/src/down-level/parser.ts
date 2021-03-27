import { CstParser } from 'chevrotain';

import * as lexer from './lexer';

class HLParserClass extends CstParser {
  DomainDeclaration = this.RULE('DomainDeclaration', () => {
    this.CONSUME(lexer.KeywordDomain);
    this.CONSUME(lexer.Identifier);
    this.CONSUME(lexer.Equal);
    this.AT_LEAST_ONE_SEP({
      SEP: lexer.Or,
      DEF: () => {
        this.SUBRULE(this.DomainElement);
      },
    });
  });

  DomainElement = this.RULE('DomainElement', () => {
    this.OR([
      {
        ALT: () => {
          this.CONSUME2(lexer.Identifier);
          this.CONSUME(lexer.ParenthesisLeft);
          this.AT_LEAST_ONE_SEP({
            SEP: lexer.Comma,
            DEF: () => {
              this.CONSUME3(lexer.Identifier);
            },
          });
          this.CONSUME(lexer.ParenthesisRight);
          this.CONSUME(lexer.KeywordWhere);
          this.AT_LEAST_ONE_SEP2({
            SEP: lexer.Comma,
            DEF: () => {
              this.SUBRULE(this.DomainValues);
            },
          });
        },
      },
      {
        ALT: () => {
          this.CONSUME(lexer.Identifier);
        },
      },
    ]);
  });

  DomainValues = this.RULE('DomainValues', () => {
    this.CONSUME(lexer.Identifier);
    this.CONSUME(lexer.KeywordIn);
    this.OR([
      {
        ALT: () => {
          this.CONSUME2(lexer.Identifier);
          this.CONSUME(lexer.DotDot);
          this.CONSUME3(lexer.Identifier);
        },
      },
      {
        ALT: () => {
          this.CONSUME(lexer.BraceLeft);
          this.AT_LEAST_ONE_SEP({
            SEP: lexer.Comma,
            DEF: () => {
              this.CONSUME4(lexer.Identifier);
            },
          });
          this.CONSUME(lexer.BraceRight);
        },
      },
    ]);
  });

  GameDeclaration = this.RULE('GameDeclaration', () => {
    this.MANY(() => {
      this.OR([{ ALT: () => this.SUBRULE(this.DomainDeclaration) }]);
    });
  });

  constructor() {
    super(lexer.tokens, { maxLookahead: 2 });
    this.performSelfAnalysis();
  }
}

export default new HLParserClass();
