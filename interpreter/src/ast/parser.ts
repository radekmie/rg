import { CstParser } from 'chevrotain';

import * as lexer from './lexer';

class RGParserClass extends CstParser {
  constDeclaration = this.RULE('constDeclaration', () => {
    this.CONSUME(lexer.KeywordConst);
    this.SUBRULE(this.identifier);
    this.CONSUME(lexer.Colon);
    this.SUBRULE(this.type);
    this.CONSUME(lexer.Equal);
    this.SUBRULE(this.value);
    this.CONSUME(lexer.Semicolon);
  });

  edgeDeclaration = this.RULE('edgeDeclaration', () => {
    this.SUBRULE(this.identifier);
    this.CONSUME(lexer.Comma);
    this.SUBRULE2(this.identifier);
    this.CONSUME(lexer.Colon);
    this.SUBRULE(this.edgeLabel);
    this.CONSUME(lexer.Semicolon);
  });

  edgeLabel = this.RULE('edgeLabel', () => {
    this.OPTION(() => {
      this.OR([
        {
          ALT: () => {
            this.SUBRULE3(this.expression);
            this.OR2([
              { ALT: () => this.CONSUME(lexer.BangEqual) },
              { ALT: () => this.CONSUME(lexer.Equal) },
              { ALT: () => this.CONSUME(lexer.EqualEqual) },
            ]);
            this.SUBRULE4(this.expression);
          },
        },
        {
          ALT: () => {
            this.SUBRULE(this.identifier);
            this.CONSUME(lexer.KeywordMode);
            this.CONSUME(lexer.Arrow);
            this.SUBRULE2(this.identifier);
          },
        },
      ]);
    });
  });

  expression = this.RULE('expression', () => {
    this.SUBRULE(this.identifier);
    this.OPTION(() => {
      this.OR([
        {
          ALT: () => {
            this.AT_LEAST_ONE(() => {
              this.CONSUME(lexer.BracketLeft);
              this.SUBRULE(this.expression);
              this.CONSUME(lexer.BracketRight);
            });
          },
        },
        {
          ALT: () => {
            this.CONSUME(lexer.ParenthesisLeft);
            this.SUBRULE2(this.expression);
            this.CONSUME(lexer.ParenthesisRight);
          },
        },
      ]);
    });
  });

  game = this.RULE('game', () => {
    this.MANY(() => {
      this.OR([
        { ALT: () => this.SUBRULE(this.constDeclaration) },
        { ALT: () => this.SUBRULE(this.edgeDeclaration) },
        { ALT: () => this.SUBRULE(this.typeDeclaration) },
        { ALT: () => this.SUBRULE(this.varDeclaration) },
      ]);
    });
  });

  identifier = this.RULE('identifier', () => {
    this.CONSUME(lexer.Identifier);
  });

  type = this.RULE('type', () => {
    this.OR([
      {
        ALT: () => {
          this.SUBRULE(this.identifier);
          this.OPTION(() => {
            this.CONSUME(lexer.Arrow);
            this.SUBRULE(this.type);
          });
        },
      },
      {
        ALT: () => {
          this.CONSUME(lexer.BraceLeft);
          this.MANY_SEP({
            SEP: lexer.Comma,
            DEF: () => {
              this.SUBRULE2(this.identifier);
            },
          });
          this.CONSUME(lexer.BraceRight);
        },
      },
    ]);
  });

  typeDeclaration = this.RULE('typeDeclaration', () => {
    this.CONSUME(lexer.KeywordType);
    this.SUBRULE(this.identifier);
    this.CONSUME(lexer.Equal);
    this.SUBRULE(this.type);
    this.CONSUME(lexer.Semicolon);
  });

  value = this.RULE('value', () => {
    this.OR([
      {
        ALT: () => {
          this.SUBRULE(this.identifier);
        },
      },
      {
        ALT: () => {
          this.CONSUME(lexer.BraceLeft);
          this.MANY_SEP({
            SEP: lexer.Comma,
            DEF: () => {
              this.SUBRULE(this.valueEntry);
            },
          });
          this.CONSUME(lexer.BraceRight);
        },
      },
    ]);
  });

  valueEntry = this.RULE('valueEntry', () => {
    this.OPTION(() => {
      this.SUBRULE(this.identifier);
    });
    this.CONSUME(lexer.Colon);
    this.SUBRULE(this.value);
  });

  varDeclaration = this.RULE('varDeclaration', () => {
    this.CONSUME(lexer.KeywordVar);
    this.SUBRULE(this.identifier);
    this.CONSUME(lexer.Colon);
    this.SUBRULE(this.type);
    this.CONSUME(lexer.Equal);
    this.SUBRULE(this.value);
    this.CONSUME(lexer.Semicolon);
  });

  constructor() {
    super(lexer.tokens, { maxLookahead: 2 });
    this.performSelfAnalysis();
  }
}

export default new RGParserClass();
