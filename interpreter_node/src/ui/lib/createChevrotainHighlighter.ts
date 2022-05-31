import { RangeSetBuilder } from '@codemirror/state';
import {
  Decoration,
  DecorationSet,
  EditorView,
  ViewPlugin,
  ViewUpdate,
} from '@codemirror/view';
import { IToken } from 'chevrotain';

const marks: Record<string, Decoration> = Object.create(null);
function mark(tag: string) {
  if (!(tag in marks)) {
    marks[tag] = Decoration.mark({ attributes: { class: `ͼ${tag}` } });
  }

  return marks[tag];
}

type ParseFunction = (source: string) => {
  color1?: string[];
  color2?: string[];
  color3?: string[];
  tokens: IToken[];
};

export function createChevrotainHighlighter(parse: ParseFunction) {
  function getDecorations(view: EditorView) {
    const marks: [number, number, string][] = [];
    const source = view.state.doc.sliceString(0);

    // Comments.
    const lines = source.split('\n');
    for (let index = 0; index < lines.length; ++index) {
      const line = lines[index];
      const position = line.indexOf('//');
      if (position !== -1) {
        const start = lines.slice(0, index).join('\n').length;
        marks.push([start + position, start + line.length + 1, '5']);
      }
    }

    // Lexing.
    try {
      const { color1, color2, color3, tokens } = parse(source);
      for (const {
        image,
        startOffset: start,
        tokenType: { name },
      } of tokens) {
        const tag = name.startsWith('Keyword')
          ? 'b'
          : name === 'Identifier'
          ? color1?.includes(image)
            ? 'k'
            : color2?.includes(image)
            ? 'l'
            : color3?.includes(image)
            ? 'm'
            : null
          : null;

        if (tag) {
          marks.push([start, start + image.length, tag]);
        }
      }
    } catch (error) {
      // It's fine to ignore that.
    }

    // Build decorations.
    marks.sort((x, y) => x[0] - y[0] || x[1] - y[1]);

    const builder = new RangeSetBuilder<Decoration>();
    for (const [start, end, tag] of marks) {
      builder.add(start, end, mark(tag));
    }

    return builder.finish();
  }

  return ViewPlugin.fromClass(
    class ChevrotainHighlighter {
      decorations: DecorationSet;

      constructor(view: EditorView) {
        this.decorations = getDecorations(view);
      }

      update({ docChanged, view }: ViewUpdate) {
        if (docChanged) {
          this.decorations = getDecorations(view);
        }
      }
    },
    { decorations: instance => instance.decorations },
  );
}
