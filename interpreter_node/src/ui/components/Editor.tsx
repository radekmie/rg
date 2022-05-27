import { javascript } from '@codemirror/lang-javascript';
import { json } from '@codemirror/lang-json';
import { RangeSetBuilder } from '@codemirror/state';
import {
  Decoration,
  DecorationSet,
  EditorView,
  ViewPlugin,
  ViewUpdate,
} from '@codemirror/view';
import CodeMirror, { ReactCodeMirrorProps } from '@uiw/react-codemirror';
import { Lexer } from 'chevrotain';

import lexerLL from '../../cst/lexer';
import lexerHL from '../../down-level/lexer';
import * as styles from './Application.module.css';
import { Autosize } from './Autosize';

const marks: Record<string, Decoration> = Object.create(null);
function mark(tag: string) {
  if (!(tag in marks)) {
    marks[tag] = Decoration.mark({ attributes: { class: `ͼ${tag}` } });
  }

  return marks[tag];
}

function createChevrotainHighlighter(lexer: Lexer) {
  return ViewPlugin.fromClass(
    class {
      decorations: DecorationSet;

      constructor(view: EditorView) {
        this.decorations = this.getDecorations(view);
      }

      update({ docChanged, view }: ViewUpdate) {
        if (docChanged) {
          this.decorations = this.getDecorations(view);
        }
      }

      getDecorations(view: EditorView) {
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

        // Tokens.
        const { tokens } = lexer.tokenize(source);
        for (const {
          image,
          startOffset: start,
          tokenType: { name },
        } of tokens) {
          const tag = name.startsWith('Keyword')
            ? 'b'
            : image === String(parseFloat(image))
            ? 'd'
            : name === 'Identifier' && image[0] === image[0].toUpperCase()
            ? 'l'
            : undefined;
          if (tag) {
            marks.push([start, start + image.length, tag]);
          }
        }

        // Build decorations.
        marks.sort((x, y) => x[0] - y[0] || x[1] - y[1]);

        const builder = new RangeSetBuilder<Decoration>();
        for (const [start, end, tag] of marks) {
          builder.add(start, end, mark(tag));
        }

        return builder.finish();
      }
    },
    { decorations: instance => instance.decorations },
  );
}

const modeToExtensions = {
  hrg: [createChevrotainHighlighter(lexerHL)],
  javascript: [javascript()],
  json: [json()],
  rg: [createChevrotainHighlighter(lexerLL)],
  text: [],
};

export type EditorProps = ReactCodeMirrorProps & {
  mode: keyof typeof modeToExtensions;
};

export function Editor({ mode, ...props }: EditorProps) {
  return (
    <Autosize>
      {({ height }) => (
        <section className={styles.wrapHidden}>
          <CodeMirror
            extensions={modeToExtensions[mode]}
            height={`${height}px`}
            {...props}
          />
        </section>
      )}
    </Autosize>
  );
}
