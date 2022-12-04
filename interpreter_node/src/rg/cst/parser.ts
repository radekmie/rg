import { CstParser } from 'chevrotain';

import * as tokens from './tokens';

class ParserClass extends CstParser {
  ConstantDeclaration = this.RULE('ConstantDeclaration', () => {
    this.CONSUME(tokens.KeywordConst);
    this.CONSUME(tokens.Identifier);
    this.CONSUME(tokens.Colon);
    this.SUBRULE(this.Type);
    this.CONSUME(tokens.Equal);
    this.SUBRULE(this.Value);
    this.CONSUME(tokens.Semicolon);
  });

  EdgeDeclaration = this.RULE('EdgeDeclaration', () => {
    this.SUBRULE(this.EdgeName);
    this.CONSUME(tokens.Comma);
    this.SUBRULE2(this.EdgeName);
    this.CONSUME(tokens.Colon);
    this.SUBRULE(this.EdgeLabel);
    this.CONSUME(tokens.Semicolon);
  });

  EdgeLabel = this.RULE('EdgeLabel', () => {
    this.OPTION(() => {
      this.OR([
        {
          ALT: () => {
            this.SUBRULE3(this.Expression);
            this.OR2([
              { ALT: () => this.CONSUME(tokens.BangEqual) },
              { ALT: () => this.CONSUME(tokens.Equal) },
              { ALT: () => this.CONSUME(tokens.EqualEqual) },
            ]);
            this.SUBRULE4(this.Expression);
          },
        },
        {
          ALT: () => {
            this.OR3([
              { ALT: () => this.CONSUME(tokens.Bang) },
              { ALT: () => this.CONSUME(tokens.Question) },
            ]);
            this.CONSUME(tokens.Identifier);
            this.CONSUME(tokens.Arrow);
            this.CONSUME2(tokens.Identifier);
          },
        },
      ]);
    });
  });

  EdgeName = this.RULE('EdgeName', () => {
    this.AT_LEAST_ONE(() => this.SUBRULE(this.EdgeNamePart));
  });

  EdgeNamePart = this.RULE('EdgeNamePart', () => {
    this.OR([
      { ALT: () => this.CONSUME(tokens.Identifier) },
      {
        ALT: () => {
          this.CONSUME(tokens.ParenthesisLeft);
          this.CONSUME2(tokens.Identifier);
          this.CONSUME(tokens.Colon);
          this.SUBRULE(this.Type);
          this.CONSUME(tokens.ParenthesisRight);
        },
      },
    ]);
  });

  Expression = this.RULE('Expression', () => {
    this.CONSUME(tokens.Identifier);
    this.OPTION(() => {
      this.CONSUME(tokens.ParenthesisLeft);
      this.SUBRULE(this.Expression);
      this.CONSUME(tokens.ParenthesisRight);
    });
    this.MANY(() => {
      this.CONSUME(tokens.BracketLeft);
      this.SUBRULE2(this.Expression);
      this.CONSUME(tokens.BracketRight);
    });
  });

  GameDeclaration = this.RULE('GameDeclaration', () => {
    this.MANY(() =>
      this.OR([
        { ALT: () => this.SUBRULE(this.ConstantDeclaration) },
        { ALT: () => this.SUBRULE(this.EdgeDeclaration) },
        { ALT: () => this.SUBRULE(this.TypeDeclaration) },
        { ALT: () => this.SUBRULE(this.VariableDeclaration) },
      ]),
    );
  });

  Type = this.RULE('Type', () => {
    this.OR([
      {
        ALT: () => {
          this.CONSUME(tokens.Identifier);
          this.OPTION(() => {
            this.CONSUME(tokens.Arrow);
            this.SUBRULE(this.Type);
          });
        },
      },
      {
        ALT: () => {
          this.CONSUME(tokens.BraceLeft);
          this.MANY_SEP({
            SEP: tokens.Comma,
            DEF: () => this.CONSUME2(tokens.Identifier),
          });
          this.CONSUME(tokens.BraceRight);
        },
      },
    ]);
  });

  TypeDeclaration = this.RULE('TypeDeclaration', () => {
    this.CONSUME(tokens.KeywordType);
    this.CONSUME(tokens.Identifier);
    this.CONSUME(tokens.Equal);
    this.SUBRULE(this.Type);
    this.CONSUME(tokens.Semicolon);
  });

  Value = this.RULE('Value', () => {
    this.OR([
      { ALT: () => this.CONSUME(tokens.Identifier) },
      {
        ALT: () => {
          this.CONSUME(tokens.BraceLeft);
          this.MANY_SEP({
            SEP: tokens.Comma,
            DEF: () => this.SUBRULE(this.ValueEntry),
          });
          this.CONSUME(tokens.BraceRight);
        },
      },
    ]);
  });

  ValueEntry = this.RULE('ValueEntry', () => {
    this.OPTION(() => this.CONSUME(tokens.Identifier));
    this.CONSUME(tokens.Colon);
    this.SUBRULE(this.Value);
  });

  VariableDeclaration = this.RULE('VariableDeclaration', () => {
    this.CONSUME(tokens.KeywordVar);
    this.CONSUME(tokens.Identifier);
    this.CONSUME(tokens.Colon);
    this.SUBRULE(this.Type);
    this.CONSUME(tokens.Equal);
    this.SUBRULE(this.Value);
    this.CONSUME(tokens.Semicolon);
  });

  constructor() {
    super(tokens.tokens, { maxLookahead: 1 });
    this.performSelfAnalysis();
  }
}

export const parser = new ParserClass();
