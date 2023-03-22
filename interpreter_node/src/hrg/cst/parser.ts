import { CstParser } from 'chevrotain';

import * as tokens from './tokens';

class Parser extends CstParser {
  AutomatonBranch = this.RULE('AutomatonBranch', () => {
    this.MANY(() => this.SUBRULE(this.AutomatonStatement));
  });

  AutomatonFunction = this.RULE('AutomatonFunction', () => {
    this.CONSUME(tokens.KeywordGraph);
    this.CONSUME(tokens.Identifier);
    this.CONSUME(tokens.ParenthesisLeft);
    this.MANY_SEP({
      SEP: tokens.Comma,
      DEF: () => this.SUBRULE(this.AutomatonFunctionArgument),
    });
    this.CONSUME(tokens.ParenthesisRight);
    this.CONSUME(tokens.BraceLeft);
    this.MANY(() => this.SUBRULE(this.AutomatonStatement));
    this.CONSUME(tokens.BraceRight);
  });

  AutomatonFunctionArgument = this.RULE('AutomatonFunctionArgument', () => {
    this.CONSUME(tokens.Identifier);
    this.CONSUME(tokens.Colon);
    this.SUBRULE(this.Type);
  });

  AutomatonStatement = this.RULE('AutomatonStatement', () => {
    this.OR([
      {
        ALT: () => {
          this.CONSUME(tokens.Identifier);
          this.CONSUME(tokens.ParenthesisLeft);
          this.MANY_SEP({
            SEP: tokens.Comma,
            DEF: () => this.SUBRULE(this.Expression),
          });
          this.CONSUME(tokens.ParenthesisRight);
        },
      },
      {
        ALT: () => {
          this.CONSUME2(tokens.Identifier);
          this.MANY(() => {
            this.CONSUME(tokens.BracketLeft);
            this.SUBRULE2(this.Expression);
            this.CONSUME(tokens.BracketRight);
          });
          this.CONSUME(tokens.Equal);
          this.SUBRULE3(this.Expression);
        },
      },
      {
        ALT: () => {
          this.CONSUME(tokens.KeywordAny);
          this.CONSUME(tokens.BraceLeft);
          this.MANY2(() => this.SUBRULE(this.AutomatonStatement));
          this.CONSUME(tokens.BraceRight);
        },
      },
      {
        ALT: () => {
          this.CONSUME(tokens.KeywordBranch);
          this.MANY_SEP2({
            SEP: tokens.KeywordOr,
            DEF: () => {
              this.CONSUME2(tokens.BraceLeft);
              this.SUBRULE(this.AutomatonBranch);
              this.CONSUME2(tokens.BraceRight);
            },
          });
        },
      },
      {
        ALT: () => {
          this.CONSUME(tokens.KeywordWhen);
          this.SUBRULE4(this.Expression);
          this.CONSUME3(tokens.BraceLeft);
          this.MANY3(() => this.SUBRULE2(this.AutomatonStatement));
          this.CONSUME3(tokens.BraceRight);
        },
      },
      {
        ALT: () => {
          this.CONSUME(tokens.KeywordLoop);
          this.CONSUME4(tokens.BraceLeft);
          this.MANY4(() => this.SUBRULE3(this.AutomatonStatement));
          this.CONSUME4(tokens.BraceRight);
        },
      },
      {
        ALT: () => {
          this.CONSUME(tokens.KeywordWhile);
          this.CONSUME3(tokens.ParenthesisLeft);
          this.SUBRULE5(this.Expression);
          this.CONSUME3(tokens.ParenthesisRight);
          this.CONSUME5(tokens.BraceLeft);
          this.MANY5(() => this.SUBRULE4(this.AutomatonStatement));
          this.CONSUME5(tokens.BraceRight);
        },
      },
    ]);
  });

  DomainDeclaration = this.RULE('DomainDeclaration', () => {
    this.CONSUME(tokens.KeywordDomain);
    this.CONSUME(tokens.Identifier);
    this.CONSUME(tokens.Equal);
    this.AT_LEAST_ONE_SEP({
      SEP: tokens.Or,
      DEF: () => this.SUBRULE(this.DomainElement),
    });
  });

  DomainElement = this.RULE('DomainElement', () => {
    this.OR([
      {
        ALT: () => {
          this.CONSUME(tokens.Identifier);
          this.CONSUME(tokens.ParenthesisLeft);
          this.AT_LEAST_ONE_SEP({
            SEP: tokens.Comma,
            DEF: () => this.CONSUME3(tokens.Identifier),
          });
          this.CONSUME(tokens.ParenthesisRight);
          this.CONSUME(tokens.KeywordWhere);
          this.AT_LEAST_ONE_SEP2({
            SEP: tokens.Comma,
            DEF: () => this.SUBRULE(this.DomainValues),
          });
        },
      },
      { ALT: () => this.CONSUME2(tokens.Identifier) },
    ]);
  });

  DomainValues = this.RULE('DomainValues', () => {
    this.CONSUME(tokens.Identifier);
    this.CONSUME(tokens.KeywordIn);
    this.OR([
      {
        ALT: () => {
          this.CONSUME2(tokens.Identifier);
          this.CONSUME(tokens.DotDot);
          this.CONSUME3(tokens.Identifier);
        },
      },
      {
        ALT: () => {
          this.CONSUME(tokens.BraceLeft);
          this.AT_LEAST_ONE_SEP({
            SEP: tokens.Comma,
            DEF: () => this.CONSUME4(tokens.Identifier),
          });
          this.CONSUME(tokens.BraceRight);
        },
      },
    ]);
  });

  Expression = this.RULE('Expression', () => {
    this.OR([
      {
        ALT: () => {
          this.CONSUME(tokens.KeywordIf);
          this.SUBRULE(this.Expression);
          this.CONSUME(tokens.KeywordThen);
          this.SUBRULE2(this.Expression);
          this.CONSUME(tokens.KeywordElse);
          this.SUBRULE3(this.Expression);
        },
      },
      {
        ALT: () => {
          this.CONSUME(tokens.BraceLeft);
          this.SUBRULE(this.Pattern);
          this.CONSUME(tokens.Equal);
          this.SUBRULE4(this.Expression);
          this.CONSUME(tokens.KeywordWhere);
          this.AT_LEAST_ONE_SEP2({
            SEP: tokens.Comma,
            DEF: () => this.SUBRULE(this.DomainValues),
          });
          this.CONSUME(tokens.BraceRight);
        },
      },
      {
        ALT: () => {
          this.SUBRULE(this.Expression2);
          this.OPTION({
            DEF: () => {
              this.CONSUME(tokens.OrOr);
              this.SUBRULE5(this.Expression);
            },
          });
        },
      },
    ]);
  });

  Expression2 = this.RULE('Expression2', () => {
    this.SUBRULE(this.Expression3);
    this.OPTION({
      DEF: () => {
        this.CONSUME(tokens.AndAnd);
        this.SUBRULE2(this.Expression2);
      },
    });
  });

  Expression3 = this.RULE('Expression3', () => {
    this.SUBRULE(this.Expression4);
    this.OPTION({
      DEF: () => {
        this.OR2([
          { ALT: () => this.CONSUME(tokens.EqualEqual) },
          { ALT: () => this.CONSUME(tokens.Gt) },
          { ALT: () => this.CONSUME(tokens.GtEqual) },
          { ALT: () => this.CONSUME(tokens.Lt) },
          { ALT: () => this.CONSUME(tokens.LtEqual) },
          { ALT: () => this.CONSUME(tokens.NotEqual) },
        ]);
        this.SUBRULE2(this.Expression3);
      },
    });
  });

  Expression4 = this.RULE('Expression4', () => {
    this.SUBRULE(this.Expression5);
    this.OPTION({
      DEF: () => {
        this.OR([
          { ALT: () => this.CONSUME(tokens.Minus) },
          { ALT: () => this.CONSUME(tokens.Plus) },
        ]);
        this.SUBRULE2(this.Expression4);
      },
    });
  });

  Expression5 = this.RULE('Expression5', () => {
    this.OR([
      {
        ALT: () => {
          this.CONSUME(tokens.Identifier);
          this.MANY(() => this.SUBRULE(this.ExpressionSuffix));
        },
      },
      {
        ALT: () => {
          this.CONSUME(tokens.ParenthesisLeft);
          this.SUBRULE(this.Expression);
          this.CONSUME(tokens.ParenthesisRight);
        },
      },
    ]);
  });

  ExpressionSuffix = this.RULE('ExpressionSuffix', () => {
    this.OR([
      {
        ALT: () => {
          this.CONSUME(tokens.BracketLeft);
          this.SUBRULE(this.Expression);
          this.CONSUME(tokens.BracketRight);
        },
      },
      {
        ALT: () => {
          this.CONSUME(tokens.ParenthesisLeft);
          this.MANY_SEP({
            SEP: tokens.Comma,
            DEF: () => this.SUBRULE2(this.Expression),
          });
          this.CONSUME(tokens.ParenthesisRight);
        },
      },
    ]);
  });

  FunctionCase = this.RULE('FunctionCase', () => {
    this.CONSUME(tokens.Identifier);
    this.CONSUME(tokens.ParenthesisLeft);
    this.MANY_SEP({
      SEP: tokens.Comma,
      DEF: () => this.SUBRULE(this.Pattern),
    });
    this.CONSUME(tokens.ParenthesisRight);
    this.CONSUME(tokens.Equal);
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
          this.CONSUME(tokens.Identifier);
          this.CONSUME(tokens.ParenthesisLeft);
          this.MANY_SEP({
            SEP: tokens.Comma,
            DEF: () => this.SUBRULE(this.Pattern),
          });
          this.CONSUME(tokens.ParenthesisRight);
        },
      },
      { ALT: () => this.CONSUME(tokens.KeywordWildcard) },
      { ALT: () => this.CONSUME2(tokens.Identifier) },
    ]);
  });

  Type = this.RULE('Type', () => {
    this.OR([
      {
        ALT: () => {
          this.CONSUME(tokens.Identifier);
          this.CONSUME(tokens.MinusGt);
          this.SUBRULE(this.Type);
        },
      },
      { ALT: () => this.CONSUME2(tokens.Identifier) },
    ]);
  });

  TypeDeclaration = this.RULE('TypeDeclaration', () => {
    this.CONSUME(tokens.Identifier);
    this.CONSUME(tokens.Colon);
    this.SUBRULE(this.Type);
  });

  VariableAssignment = this.RULE('VariableAssignment', () => {
    this.CONSUME(tokens.Identifier);
    this.CONSUME(tokens.Equal);
    this.SUBRULE(this.Expression);
  });

  constructor() {
    super(tokens.tokens, { maxLookahead: 2 });
    this.performSelfAnalysis();
  }
}

export const parser = new Parser();
