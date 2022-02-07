import { CstParser } from 'chevrotain';

import * as lexer from './lexer';

class HLParserClass extends CstParser {
  AutomatonBranch = this.RULE('AutomatonBranch', () => {
    this.MANY({
      DEF: () => {
        this.SUBRULE(this.AutomatonStatement);
      },
    });
  });

  AutomatonFunction = this.RULE('AutomatonFunction', () => {
    this.CONSUME(lexer.KeywordGraph);
    this.CONSUME(lexer.Identifier);
    this.CONSUME(lexer.ParenthesisLeft);
    this.MANY_SEP({
      SEP: lexer.Comma,
      DEF: () => {
        this.CONSUME2(lexer.Identifier);
      },
    });
    this.CONSUME(lexer.ParenthesisRight);
    this.CONSUME(lexer.BraceLeft);
    this.MANY({
      DEF: () => {
        this.SUBRULE(this.AutomatonStatement);
      },
    });
    this.CONSUME(lexer.BraceRight);
  });

  AutomatonStatement = this.RULE('AutomatonStatement', () => {
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
          this.MANY(() => {
            this.CONSUME(lexer.BracketLeft);
            this.SUBRULE2(this.Expression);
            this.CONSUME(lexer.BracketRight);
          });
          this.CONSUME(lexer.Equal);
          this.SUBRULE3(this.Expression);
        },
      },
      {
        ALT: () => {
          this.CONSUME(lexer.KeywordBranch);
          this.MANY_SEP2({
            SEP: lexer.KeywordOr,
            DEF: () => {
              this.CONSUME(lexer.BraceLeft);
              this.SUBRULE(this.AutomatonBranch);
              this.CONSUME(lexer.BraceRight);
            },
          });
        },
      },
      {
        ALT: () => {
          this.CONSUME(lexer.KeywordIf);
          this.CONSUME2(lexer.ParenthesisLeft);
          this.SUBRULE(this.Condition);
          this.CONSUME2(lexer.ParenthesisRight);
          this.CONSUME2(lexer.BraceLeft);
          this.MANY2({
            DEF: () => {
              this.SUBRULE(this.AutomatonStatement);
            },
          });
          this.CONSUME2(lexer.BraceRight);
        },
      },
      {
        ALT: () => {
          this.CONSUME(lexer.KeywordWhile);
          this.CONSUME3(lexer.ParenthesisLeft);
          this.SUBRULE2(this.Condition);
          this.CONSUME3(lexer.ParenthesisRight);
          this.CONSUME3(lexer.BraceLeft);
          this.MANY3({
            DEF: () => {
              this.SUBRULE2(this.AutomatonStatement);
            },
          });
          this.CONSUME3(lexer.BraceRight);
        },
      },
    ]);
  });

  Condition = this.RULE('Condition', () => {
    this.SUBRULE(this.ExpressionSimple);
    this.OR([
      { ALT: () => this.CONSUME(lexer.EqualEqual) },
      { ALT: () => this.CONSUME(lexer.Gt) },
      { ALT: () => this.CONSUME(lexer.GtEqual) },
      { ALT: () => this.CONSUME(lexer.Lt) },
      { ALT: () => this.CONSUME(lexer.LtEqual) },
      { ALT: () => this.CONSUME(lexer.OrOr) },
    ]);
    this.SUBRULE2(this.ExpressionSimple);
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
          this.SUBRULE(this.Expression);
          this.CONSUME(lexer.KeywordElse);
          this.SUBRULE2(this.Expression);
        },
      },
      {
        ALT: () => {
          this.CONSUME(lexer.BraceLeft);
          this.SUBRULE(this.Pattern);
          this.CONSUME(lexer.Equal);
          this.SUBRULE3(this.Expression);
          this.CONSUME(lexer.KeywordWhere);
          this.AT_LEAST_ONE_SEP2({
            SEP: lexer.Comma,
            DEF: () => {
              this.SUBRULE(this.DomainValues);
            },
          });
          this.CONSUME(lexer.BraceRight);
        },
      },
      {
        ALT: () => {
          this.SUBRULE(this.ExpressionSimple);
          this.OPTION({
            DEF: () => {
              this.OR2([
                { ALT: () => this.CONSUME(lexer.EqualEqual) },
                { ALT: () => this.CONSUME(lexer.Minus) },
                { ALT: () => this.CONSUME(lexer.Plus) },
              ]);
              this.SUBRULE2(this.ExpressionSimple);
            },
          });
        },
      },
    ]);
  });

  ExpressionInParenthesis = this.RULE('ExpressionInParenthesis', () => {
    this.CONSUME(lexer.ParenthesisLeft);
    this.MANY_SEP({
      SEP: lexer.Comma,
      DEF: () => {
        this.SUBRULE(this.Expression);
      },
    });
    this.CONSUME(lexer.ParenthesisRight);
  });

  ExpressionSimple = this.RULE('ExpressionSimple', () => {
    this.OR([
      {
        ALT: () => {
          this.CONSUME(lexer.Identifier);
          this.AT_LEAST_ONE({
            DEF: () => {
              this.SUBRULE(this.ExpressionInParenthesis);
            },
          });
        },
      },
      {
        ALT: () => {
          this.CONSUME2(lexer.Identifier);
        },
      },
      {
        ALT: () => {
          this.CONSUME(lexer.ParenthesisLeft);
          this.SUBRULE(this.Expression);
          this.CONSUME(lexer.ParenthesisRight);
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

  GameDeclaration = this.RULE('GameDeclaration', () => {
    this.MANY(() => {
      this.OR([
        { ALT: () => this.SUBRULE(this.AutomatonFunction) },
        { ALT: () => this.SUBRULE(this.DomainDeclaration) },
        { ALT: () => this.SUBRULE(this.FunctionCase) },
        { ALT: () => this.SUBRULE(this.TypeDeclaration) },
        { ALT: () => this.SUBRULE(this.VariableAssignment) },
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

  Type = this.RULE('Type', () => {
    this.OR([
      {
        ALT: () => {
          this.CONSUME(lexer.Identifier);
          this.CONSUME(lexer.MinusGt);
          this.SUBRULE(this.Type);
        },
      },
      {
        ALT: () => {
          this.CONSUME2(lexer.Identifier);
        },
      },
    ]);
  });

  TypeDeclaration = this.RULE('TypeDeclaration', () => {
    this.CONSUME(lexer.Identifier);
    this.CONSUME(lexer.Colon);
    this.SUBRULE(this.Type);
  });

  VariableAssignment = this.RULE('VariableAssignment', () => {
    this.CONSUME(lexer.Identifier);
    this.CONSUME(lexer.Equal);
    this.SUBRULE(this.Expression);
  });

  constructor() {
    super(lexer.tokens, { maxLookahead: 2 });
    this.performSelfAnalysis();
  }
}

export default new HLParserClass();
