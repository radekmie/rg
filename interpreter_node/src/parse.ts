import * as hrg from './hrg';
import * as rg from './rg';
import * as translators from './translators';
import { Extension, Settings } from './types';

function analyze(source: string, extension: Extension) {
  switch (extension) {
    case Extension.hrg: {
      const sourceHrg = source;
      const cstHrg = hrg.cst.parse(sourceHrg);
      const astHrg = hrg.ast.visit(cstHrg);
      const astRg = translators.hrg2rg(astHrg);
      const sourceRg = rg.ast.serializeGameDeclaration(astRg);
      const cstRg = rg.cst.parse(sourceRg);
      return { astHrg, astRg, cstHrg, cstRg, sourceHrg, sourceRg };
    }
    case Extension.rg: {
      const sourceHrg = null;
      const cstHrg = null;
      const astHrg = null;
      const sourceRg = source;
      const cstRg = rg.cst.parse(sourceRg);
      const astRg = rg.ast.visit(cstRg);
      return { astHrg, astRg, cstHrg, cstRg, sourceHrg, sourceRg };
    }
  }
}

export function parse(content: string, settings: Settings) {
  const { astHrg, astRg, cstHrg, cstRg, sourceHrg, sourceRg } = analyze(
    content,
    settings.extension,
  );

  if (settings.flags.compactSkipEdges) {
    rg.optimizer.compactSkipEdges(astRg);
  }

  const graphvizRg = rg.ast.graphviz(astRg);
  const istRg = rg.ist.build(astRg);

  return {
    astHrg,
    astRg,
    cstHrg,
    cstRg,
    graphvizRg,
    istRg,
    sourceHrg,
    sourceRg,
  };
}
