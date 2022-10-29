import * as utils from '../../utils';
import * as ast from '../ast';

export function expandGeneratorNodes(gameDeclaration: ast.GameDeclaration) {
  for (const edge of gameDeclaration.edges.slice()) {
    if (ast.lib.hasBindings(edge.lhs) || ast.lib.hasBindings(edge.rhs)) {
      utils.remove(gameDeclaration.edges, edge);

      const bindings = [
        ...ast.lib.bindings(edge.lhs),
        ...ast.lib.bindings(edge.rhs),
      ].reduce<ast.Binding[]>(utils.unique, []);

      for (const mapping of createMappings(gameDeclaration, bindings)) {
        gameDeclaration.edges.push(
          ast.EdgeDeclaration({
            lhs: substituteBindings(edge.lhs, mapping),
            rhs: substituteBindings(edge.rhs, mapping),
            label: ast.lib.substituteInEdgeLabel(edge.label, mapping),
          }),
        );
      }
    }
  }
}

function createMappings(
  gameDeclaration: ast.GameDeclaration,
  bindings: ast.Binding[],
) {
  return bindings
    .map(binding =>
      ast.lib.typeValues(gameDeclaration, binding.type).map(ast.serializeValue),
    )
    .reduce<string[][]>(utils.cartesian, [[]])
    .map(values =>
      utils.mapToObject(bindings, (binding, index) => [
        binding.identifier,
        ast.Reference({ identifier: values[index] }),
      ]),
    );
}

function substituteBindings(
  edgeName: ast.EdgeName,
  mapping: Record<string, ast.Reference>,
) {
  if (!ast.lib.hasBindings) {
    return edgeName;
  }

  const identifier = edgeName.parts
    .map(part => {
      switch (part.kind) {
        case 'Binding':
          return mapping[part.identifier].identifier;
        case 'Literal':
          return part.identifier;
      }
    })
    .join('__bind__');

  return ast.EdgeName({ parts: [ast.Literal({ identifier })] });
}
