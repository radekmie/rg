import { CstParser } from 'chevrotain';

import * as lexer from './lexer';

class HLParserClass extends CstParser {
  Condition = this.RULE('Condition', () => {
    this.SUBRULE(this.Expression2);
    this.CONSUME(lexer.EqualEqual);
    this.SUBRULE2(this.Expression2);
  });

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
          this.CONSUME(lexer.Identifier);
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
          this.CONSUME2(lexer.Identifier);
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

  Expression = this.RULE('Expression', () => {
    this.OR([
      {
        ALT: () => {
          this.CONSUME(lexer.KeywordIf);
          this.SUBRULE(this.Condition);
          this.CONSUME(lexer.KeywordThen);
          this.SUBRULE2(this.Expression);
          this.CONSUME(lexer.KeywordElse);
          this.SUBRULE3(this.Expression);
        },
      },
      {
        ALT: () => {
          this.SUBRULE(this.Expression2);
          this.OPTION({
            DEF: () => {
              this.OR2([
                { ALT: () => this.CONSUME(lexer.EqualEqual) },
                { ALT: () => this.CONSUME(lexer.Minus) },
                { ALT: () => this.CONSUME(lexer.Plus) },
              ]);
              this.SUBRULE2(this.Expression2);
            },
          });
        },
      },
    ]);
  });

  Expression2 = this.RULE('Expression2', () => {
    this.OR([
      {
        ALT: () => {
          this.CONSUME(lexer.Identifier);
          this.CONSUME(lexer.ParenthesisLeft);
          this.MANY_SEP({
            SEP: lexer.Comma,
            DEF: () => {
              this.SUBRULE(this.Expression);
            },
          });
          this.CONSUME(lexer.ParenthesisRight);
        },
      },
      {
        ALT: () => {
          this.CONSUME2(lexer.Identifier);
        },
      },
      {
        ALT: () => {
          this.CONSUME2(lexer.ParenthesisLeft);
          this.SUBRULE2(this.Expression);
          this.CONSUME2(lexer.ParenthesisRight);
        },
      },
    ]);
  });

  FunctionCase = this.RULE('FunctionCase', () => {
    this.CONSUME(lexer.Identifier);
    this.CONSUME(lexer.ParenthesisLeft);
    this.MANY_SEP({
      SEP: lexer.Comma,
      DEF: () => {
        this.SUBRULE(this.Pattern);
      },
    });
    this.CONSUME(lexer.ParenthesisRight);
    this.CONSUME(lexer.Equal);
    this.SUBRULE(this.Expression);
  });

  FunctionDeclaration = this.RULE('FunctionDeclaration', () => {
    this.CONSUME(lexer.Identifier);
    this.CONSUME(lexer.Colon);
    this.SUBRULE(this.FunctionType);
    this.AT_LEAST_ONE({
      // FIXME: This prevents the loop of `FunctionCase`s but it shouldn't be needed.
      GATE: () => this.LA(2).tokenType !== lexer.Colon,
      DEF: () => {
        this.SUBRULE(this.FunctionCase);
      },
    });
  });

  FunctionType = this.RULE('FunctionType', () => {
    this.OR([
      {
        ALT: () => {
          this.CONSUME(lexer.Identifier);
          this.CONSUME(lexer.MinusGt);
          this.SUBRULE(this.FunctionType);
        },
      },
      {
        ALT: () => {
          this.CONSUME2(lexer.Identifier);
        },
      },
    ]);
  });

  GameDeclaration = this.RULE('GameDeclaration', () => {
    this.MANY(() => {
      this.OR([
        { ALT: () => this.SUBRULE(this.DomainDeclaration) },
        { ALT: () => this.SUBRULE(this.FunctionDeclaration) },
      ]);
    });
  });

  Pattern = this.RULE('Pattern', () => {
    this.OR([
      {
        ALT: () => {
          this.CONSUME(lexer.Identifier);
          this.CONSUME(lexer.ParenthesisLeft);
          this.MANY_SEP({
            SEP: lexer.Comma,
            DEF: () => {
              this.SUBRULE(this.Pattern);
            },
          });
          this.CONSUME(lexer.ParenthesisRight);
        },
      },
      {
        ALT: () => {
          this.CONSUME(lexer.KeywordWildcard);
        },
      },
      {
        ALT: () => {
          this.CONSUME2(lexer.Identifier);
        },
      },
    ]);
  });

  constructor() {
    super(lexer.tokens, { maxLookahead: 2 });
    this.performSelfAnalysis();
  }
}

export default new HLParserClass();

import { writeFileSync } from 'fs';
import { createSyntaxDiagramsCode } from 'chevrotain';
writeFileSync('./grammar.html', createSyntaxDiagramsCode(new HLParserClass().getSerializedGastProductions()));
