import * as utils from '../../utils';
import * as ast from '../ast';

// FIXME: It doesn't handle the case when both verticles have the same bind.
export function expandGeneratorNodes(gameDeclaration: ast.GameDeclaration) {
  const edgeNames = gameDeclaration.edges
    .flatMap(edge => [edge.lhs, edge.rhs])
    .reduce<ast.EdgeName[]>(utils.unique, [])
    .filter(ast.lib.hasBindings);

  for (const edgeName of edgeNames) {
    const bindings = ast.lib.bindings(edgeName);
    const mappedEdgeNames = bindings
      .map(binding =>
        ast.lib
          .typeValues(gameDeclaration, binding.type)
          .map(ast.serializeValue),
      )
      .reduce<string[][]>(utils.cartesian, [[]])
      .map<[Record<string, ast.Reference>, ast.EdgeName]>(values => [
        utils.mapToObject(bindings, (binding, index) => [
          binding.identifier,
          ast.Reference({ identifier: values[index] }),
        ]),
        ast.EdgeName({
          parts: [
            ast.Literal({
              identifier: edgeName.parts
                .map(part => {
                  switch (part.kind) {
                    case 'Binding':
                      return values.shift()!;
                    case 'Literal':
                      return part.identifier;
                  }
                })
                .join('__bind__'),
            }),
          ],
        }),
      ]);

    for (const edge of gameDeclaration.edges.slice()) {
      if (utils.isEqual(edge.lhs, edgeName)) {
        utils.remove(gameDeclaration.edges, edge);
        for (const [mapping, edgeName] of mappedEdgeNames) {
          gameDeclaration.edges.push(
            ast.EdgeDeclaration({
              lhs: edgeName,
              rhs: edge.rhs,
              label: ast.lib.substituteInEdgeLabel(edge.label, mapping),
            }),
          );
        }
      }
    }

    for (const edge of gameDeclaration.edges.slice()) {
      if (utils.isEqual(edge.rhs, edgeName)) {
        utils.remove(gameDeclaration.edges, edge);
        for (const [mapping, edgeName] of mappedEdgeNames) {
          gameDeclaration.edges.push(
            ast.EdgeDeclaration({
              lhs: edge.lhs,
              rhs: edgeName,
              label: ast.lib.substituteInEdgeLabel(edge.label, mapping),
            }),
          );
        }
      }
    }
  }
}
