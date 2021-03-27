import lexer from './lexer';
import parser from './parser';
import * as ast from './types';
import visitor from './visitor';
import * as utils from '../utils';

function print(ast: ast.GameDeclaration) {
  function getElements(values: ast.DomainValues) {
    switch (values.kind) {
      case 'DomainRange': {
        const max = +values.max;
        const min = +values.min;
        return Array.from(
          { length: max - min + 1 },
          (_, index) => `${index + min}`,
        );
      }
      case 'DomainSet':
        return values.elements;
    }
  }

  const domains = ast.domains.reduce<Record<string, string[]>>(
    (domains, domain) => {
      domains[domain.identifier] = domain.elements.flatMap(element => {
        switch (element.kind) {
          case 'DomainGenerator':
            return element.args
              .map(identifier => {
                const values = utils.find(element.values, { identifier });
                if (values === undefined)
                  throw new Error(`Missing values for "${identifier}".`);
                return getElements(values);
              })
              .reduce<string[][]>(utils.cartesian, [[]])
              .map(args => `${element.identifier}__${args.join('_')}`);
          case 'DomainLiteral':
            return element.identifier in domains
              ? domains[element.identifier]
              : [element.identifier];
        }
      });
      return domains;
    },
    Object.create(null),
  );

  return [
    ...Object.entries(domains).flatMap(([domain, elements]) => [
      `type ${domain} = { ${elements.join(', ')} };`,
    ]),
  ].join('\n');
}

export default function translate(source: string) {
  const result = lexer.tokenize(source);
  if (result.errors.length > 0)
    throw Object.assign(new Error('Lexer error'), { errors: result.errors });

  parser.input = result.tokens;
  const cstNode = parser.GameDeclaration();

  if (parser.errors.length > 0)
    throw Object.assign(new Error('Parser error'), { errors: parser.errors });

  const ast: ast.GameDeclaration = visitor.visitNode(cstNode);
  const ll = print(ast);
  return ll;
}
