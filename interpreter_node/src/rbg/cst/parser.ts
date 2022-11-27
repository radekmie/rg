import { CstParser } from 'chevrotain';

import * as tokens from './tokens';

// Notation and ordering follows the original definitions from the extended
// version (https://arxiv.org/abs/1706.02462) of "Regular Boardgames" by
// Jakub Kowalski, Maksymilian Mika, Jakub Sutowicz, and Marek Szykuła.

class ParserClass extends CstParser {
  GameDescription = this.RULE('GameDescription', () => {
    this.MANY(() => {
      this.OR([
        { ALT: () => this.SUBRULE(this.PiecesSection) },
        { ALT: () => this.SUBRULE(this.VariablesSection) },
        { ALT: () => this.SUBRULE(this.PlayersSection) },
        { ALT: () => this.SUBRULE(this.BoardSection) },
        { ALT: () => this.SUBRULE(this.RulesSection) },
      ]);
    });
  });

  PieceName = this.RULE('PieceName', () => {
    this.CONSUME(tokens.Ident);
  });

  PiecesSection = this.RULE('PiecesSection', () => {
    this.CONSUME(tokens.Hash);
    this.CONSUME(tokens.KeywordPieces);
    this.CONSUME(tokens.Equal);
    this.AT_LEAST_ONE_SEP({
      SEP: tokens.Comma,
      DEF: () => this.SUBRULE(this.PieceName),
    });
  });

  VariableName = this.RULE('VariableName', () => {
    this.CONSUME(tokens.Ident);
  });

  VariableBound = this.RULE('VariableBound', () => {
    this.CONSUME(tokens.Nat);
  });

  BoundedVariable = this.RULE('BoundedVariable', () => {
    this.SUBRULE(this.VariableName);
    this.CONSUME(tokens.ParenthesisLeft);
    this.SUBRULE(this.VariableBound);
    this.CONSUME(tokens.ParenthesisRight);
  });

  VariablesSection = this.RULE('VariablesSection', () => {
    this.CONSUME(tokens.Hash);
    this.CONSUME(tokens.KeywordVariables);
    this.CONSUME(tokens.Equal);
    this.MANY_SEP({
      SEP: tokens.Comma,
      DEF: () => this.SUBRULE(this.BoundedVariable),
    });
  });

  PlayersSection = this.RULE('PlayersSection', () => {
    this.CONSUME(tokens.Hash);
    this.CONSUME(tokens.KeywordPlayers);
    this.CONSUME(tokens.Equal);
    this.AT_LEAST_ONE_SEP({
      SEP: tokens.Comma,
      DEF: () => this.SUBRULE(this.BoundedVariable),
    });
  });

  Label = this.RULE('Label', () => {
    this.CONSUME(tokens.Ident);
  });

  NodeName = this.RULE('NodeName', () => {
    this.CONSUME(tokens.Ident);
  });

  Edge = this.RULE('Edge', () => {
    this.SUBRULE(this.Label);
    this.CONSUME(tokens.Colon);
    this.SUBRULE(this.NodeName);
  });

  Node = this.RULE('Node', () => {
    this.SUBRULE(this.NodeName);
    this.CONSUME(tokens.BracketLeft);
    this.SUBRULE(this.PieceName);
    this.CONSUME(tokens.BracketRight);
    this.CONSUME(tokens.BraceLeft);
    this.AT_LEAST_ONE_SEP({
      SEP: tokens.Comma,
      DEF: () => this.SUBRULE(this.Edge),
    });
    this.CONSUME(tokens.BraceRight);
  });

  BoardSection = this.RULE('BoardSection', () => {
    this.CONSUME(tokens.Hash);
    this.CONSUME(tokens.KeywordBoard);
    this.CONSUME(tokens.Equal);
    this.AT_LEAST_ONE(() => this.SUBRULE(this.Node));
  });

  Shift = this.RULE('Shift', () => {
    this.SUBRULE(this.Label);
  });

  On = this.RULE('On', () => {
    this.CONSUME(tokens.BraceLeft);
    this.MANY_SEP({
      SEP: tokens.Comma,
      DEF: () => this.SUBRULE(this.PieceName),
    });
    this.CONSUME(tokens.BraceRight);
  });

  Off = this.RULE('Off', () => {
    this.CONSUME(tokens.BracketLeft);
    this.SUBRULE(this.PieceName);
    this.CONSUME(tokens.BracketRight);
  });

  RValue = this.RULE('RValue', () => {
    this.SUBRULE(this.Sum);
  });

  Sum = this.RULE('Sum', () => {
    this.SUBRULE(this.SumElement);
    this.MANY(() => this.SUBRULE(this.NextSumElements));
  });

  NextSumElements = this.RULE('NextSumElements', () => {
    this.OR([
      {
        ALT: () => {
          this.CONSUME(tokens.Plus);
          this.SUBRULE(this.SumElement);
        },
      },
      {
        ALT: () => {
          this.CONSUME(tokens.Dash);
          this.SUBRULE2(this.SumElement);
        },
      },
    ]);
  });

  SumElement = this.RULE('SumElement', () => {
    this.SUBRULE(this.MultiplicationElement);
    this.MANY(() => this.SUBRULE(this.NextMultiplicationElements));
  });

  NextMultiplicationElements = this.RULE('NextMultiplicationElements', () => {
    this.OR([
      {
        ALT: () => {
          this.CONSUME(tokens.Star);
          this.SUBRULE(this.MultiplicationElement);
        },
      },
      {
        ALT: () => {
          this.CONSUME(tokens.Slash);
          this.SUBRULE2(this.MultiplicationElement);
        },
      },
    ]);
  });

  MultiplicationElement = this.RULE('MultiplicationElement', () => {
    this.OR([
      {
        ALT: () => {
          this.CONSUME(tokens.ParenthesisLeft);
          this.SUBRULE(this.Sum);
          this.CONSUME(tokens.ParenthesisRight);
        },
      },
      { ALT: () => this.CONSUME(tokens.Nat) },
      { ALT: () => this.SUBRULE(this.VariableName) },
    ]);
  });

  Assignment = this.RULE('Assignment', () => {
    this.CONSUME(tokens.BracketLeftDollar);
    this.SUBRULE(this.VariableName);
    this.CONSUME(tokens.Equal);
    this.SUBRULE(this.RValue);
    this.CONSUME(tokens.BracketRight);
  });

  ComparisonOperator = this.RULE('ComparisonOperator', () => {
    this.OR([
      { ALT: () => this.CONSUME(tokens.Gt) },
      { ALT: () => this.CONSUME(tokens.GtEqual) },
      { ALT: () => this.CONSUME(tokens.EqualEqual) },
      { ALT: () => this.CONSUME(tokens.BangEqual) },
      { ALT: () => this.CONSUME(tokens.LtEqual) },
      { ALT: () => this.CONSUME(tokens.Lt) },
    ]);
  });

  Comparison = this.RULE('Comparison', () => {
    this.CONSUME(tokens.BraceLeftDollar);
    this.SUBRULE(this.RValue);
    this.SUBRULE(this.ComparisonOperator);
    this.SUBRULE2(this.RValue);
    this.CONSUME(tokens.BraceRight);
  });

  // NOTE: This is missing in the paper.
  PlayerName = this.RULE('PlayerName', () => {
    this.CONSUME(tokens.Ident);
  });

  Switch = this.RULE('Switch', () => {
    this.OR([
      {
        ALT: () => {
          this.CONSUME(tokens.DashGt);
          this.SUBRULE(this.PlayerName);
        },
      },
      { ALT: () => this.CONSUME(tokens.DashGtGt) },
    ]);
  });

  MoveCheck = this.RULE('MoveCheck', () => {
    this.OR([
      {
        ALT: () => {
          this.CONSUME(tokens.BraceLeftQuestion);
          this.SUBRULE(this.Rule);
          this.CONSUME(tokens.BraceRight);
        },
      },
      {
        ALT: () => {
          this.CONSUME2(tokens.BraceLeftBang);
          this.SUBRULE2(this.Rule);
          this.CONSUME2(tokens.BraceRight);
        },
      },
    ]);
  });

  Action = this.RULE('Action', () => {
    this.OR([
      { ALT: () => this.SUBRULE(this.Shift) },
      { ALT: () => this.SUBRULE(this.On) },
      { ALT: () => this.SUBRULE(this.Off) },
      { ALT: () => this.SUBRULE(this.Assignment) },
      { ALT: () => this.SUBRULE(this.Comparison) },
      { ALT: () => this.SUBRULE(this.Switch) },
      { ALT: () => this.SUBRULE(this.MoveCheck) },
    ]);
  });

  Rule = this.RULE('Rule', () => {
    this.SUBRULE(this.RuleSum);
  });

  RuleSum = this.RULE('RuleSum', () => {
    // NOTE: The paper says "rule-sum-element { ‘+’ rule-sum-elements }", but
    // the "rule-sum-elements" is not defined and this seems like a more
    // natural definition.
    this.AT_LEAST_ONE_SEP({
      SEP: tokens.Plus,
      DEF: () => this.SUBRULE(this.RuleSumElement),
    });
  });

  RuleSumElement = this.RULE('RuleSumElement', () => {
    this.AT_LEAST_ONE({
      DEF: () => this.SUBRULE(this.RuleConcatenationElement),
    });
  });

  RuleConcatenationElement = this.RULE('RuleConcatenationElement', () => {
    this.OR([
      {
        ALT: () => {
          this.SUBRULE(this.Action);
          this.SUBRULE(this.PotentialPower);
        },
      },
      {
        ALT: () => {
          this.CONSUME(tokens.ParenthesisLeft);
          this.SUBRULE(this.RuleSum);
          this.CONSUME(tokens.ParenthesisRight);
          this.SUBRULE2(this.PotentialPower);
        },
      },
    ]);
  });

  PotentialPower = this.RULE('PotentialPower', () => {
    this.OPTION(() => this.CONSUME(tokens.Star));
  });

  RulesSection = this.RULE('RulesSection', () => {
    this.CONSUME(tokens.Hash);
    this.CONSUME(tokens.KeywordRules);
    this.CONSUME(tokens.Equal);
    this.SUBRULE(this.Rule);
  });

  constructor() {
    super(tokens.tokens, { maxLookahead: 2 });
    this.performSelfAnalysis();
  }
}

export const parser = new ParserClass();
