import { CstParser } from 'chevrotain';

import * as lexer from './lexer';

class RGParserClass extends CstParser {
  ConstantDeclaration = this.RULE('ConstantDeclaration', () => {
    this.CONSUME(lexer.KeywordConst);
    this.CONSUME(lexer.Identifier);
    this.CONSUME(lexer.Colon);
    this.SUBRULE(this.Type);
    this.CONSUME(lexer.Equal);
    this.SUBRULE(this.Value);
    this.CONSUME(lexer.Semicolon);
  });

  EdgeDeclaration = this.RULE('EdgeDeclaration', () => {
    this.SUBRULE(this.EdgeName);
    this.CONSUME(lexer.Comma);
    this.SUBRULE2(this.EdgeName);
    this.CONSUME(lexer.Colon);
    this.SUBRULE(this.EdgeLabel);
    this.CONSUME(lexer.Semicolon);
  });

  EdgeLabel = this.RULE('EdgeLabel', () => {
    this.OPTION(() => {
      this.OR([
        {
          ALT: () => {
            this.SUBRULE3(this.Expression);
            this.OR2([
              { ALT: () => this.CONSUME(lexer.BangEqual) },
              { ALT: () => this.CONSUME(lexer.Equal) },
              { ALT: () => this.CONSUME(lexer.EqualEqual) },
            ]);
            this.SUBRULE4(this.Expression);
          },
        },
        {
          ALT: () => {
            this.CONSUME(lexer.Identifier);
            this.CONSUME(lexer.KeywordMode);
            this.CONSUME(lexer.Arrow);
            this.CONSUME2(lexer.Identifier);
          },
        },
      ]);
    });
  });

  EdgeName = this.RULE('EdgeName', () => {
    this.AT_LEAST_ONE(() => {
      this.SUBRULE(this.EdgeNamePart);
    });
  });

  EdgeNamePart = this.RULE('EdgeNamePart', () => {
    this.OR([
      {
        ALT: () => {
          this.CONSUME(lexer.Identifier);
        },
      },
      {
        ALT: () => {
          this.CONSUME(lexer.ParenthesisLeft);
          this.CONSUME2(lexer.Identifier);
          this.CONSUME(lexer.Colon);
          this.SUBRULE(this.Type);
          this.CONSUME(lexer.ParenthesisRight);
        },
      },
    ]);
  });

  Expression = this.RULE('Expression', () => {
    this.CONSUME(lexer.Identifier);
    this.OPTION(() => {
      this.OR([
        {
          ALT: () => {
            this.AT_LEAST_ONE(() => {
              this.CONSUME(lexer.BracketLeft);
              this.SUBRULE(this.Expression);
              this.CONSUME(lexer.BracketRight);
            });
          },
        },
        {
          ALT: () => {
            this.CONSUME(lexer.ParenthesisLeft);
            this.SUBRULE2(this.Expression);
            this.CONSUME(lexer.ParenthesisRight);
          },
        },
      ]);
    });
  });

  GameDeclaration = this.RULE('GameDeclaration', () => {
    this.MANY(() => {
      this.OR([
        { ALT: () => this.SUBRULE(this.ConstantDeclaration) },
        { ALT: () => this.SUBRULE(this.EdgeDeclaration) },
        { ALT: () => this.SUBRULE(this.TypeDeclaration) },
        { ALT: () => this.SUBRULE(this.VariableDeclaration) },
      ]);
    });
  });

  Type = this.RULE('Type', () => {
    this.OR([
      {
        ALT: () => {
          this.CONSUME(lexer.Identifier);
          this.OPTION(() => {
            this.CONSUME(lexer.Arrow);
            this.SUBRULE(this.Type);
          });
        },
      },
      {
        ALT: () => {
          this.CONSUME(lexer.BraceLeft);
          this.MANY_SEP({
            SEP: lexer.Comma,
            DEF: () => {
              this.CONSUME2(lexer.Identifier);
            },
          });
          this.CONSUME(lexer.BraceRight);
        },
      },
    ]);
  });

  TypeDeclaration = this.RULE('TypeDeclaration', () => {
    this.CONSUME(lexer.KeywordType);
    this.CONSUME(lexer.Identifier);
    this.CONSUME(lexer.Equal);
    this.SUBRULE(this.Type);
    this.CONSUME(lexer.Semicolon);
  });

  Value = this.RULE('Value', () => {
    this.OR([
      {
        ALT: () => {
          this.CONSUME(lexer.Identifier);
        },
      },
      {
        ALT: () => {
          this.CONSUME(lexer.BraceLeft);
          this.MANY_SEP({
            SEP: lexer.Comma,
            DEF: () => {
              this.SUBRULE(this.ValueEntry);
            },
          });
          this.CONSUME(lexer.BraceRight);
        },
      },
    ]);
  });

  ValueEntry = this.RULE('ValueEntry', () => {
    this.OPTION(() => {
      this.CONSUME(lexer.Identifier);
    });
    this.CONSUME(lexer.Colon);
    this.SUBRULE(this.Value);
  });

  VariableDeclaration = this.RULE('VariableDeclaration', () => {
    this.CONSUME(lexer.KeywordVar);
    this.CONSUME(lexer.Identifier);
    this.CONSUME(lexer.Colon);
    this.SUBRULE(this.Type);
    this.CONSUME(lexer.Equal);
    this.SUBRULE(this.Value);
    this.CONSUME(lexer.Semicolon);
  });

  constructor() {
    super(lexer.tokens, { maxLookahead: 2 });
    this.performSelfAnalysis();
  }
}

export default new RGParserClass();
