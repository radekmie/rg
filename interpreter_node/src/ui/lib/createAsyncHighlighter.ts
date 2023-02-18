import { RangeSetBuilder } from '@codemirror/state';
import {
  Decoration,
  EditorView,
  ViewPlugin,
  ViewUpdate,
} from '@codemirror/view';

const boundaryWordRegex = /\b\w+\b/g;
const commentRegex = /\/\/.*?(?:\n|$)/g;
const pragmaRegex = /@\w+\b/g;
const emptyDecorations = new RangeSetBuilder<Decoration>().finish();
const marksPerType = '5fbklm'
  .split('')
  .map(tag => Decoration.mark({ attributes: { class: `ͼ${tag}` } }));

type ParseFunction = (source: string) => Promise<Set<string>[]>;

export function createAsyncHighlighter(parse: ParseFunction) {
  async function getDecorations(view: EditorView) {
    const marks: [number, number, number][] = [];
    const source = view.state.doc.sliceString(0);

    // Comment and pragma decorations.
    for (const { index, 0: image } of source.matchAll(commentRegex)) {
      if (index !== undefined) {
        marks.push([index, index + image.length, 0]);
      }
    }

    for (const { index, 0: image } of source.matchAll(pragmaRegex)) {
      if (index !== undefined) {
        marks.push([index, index + image.length, 1]);
      }
    }

    // Parse tokens.
    let parseResult: Awaited<ReturnType<ParseFunction>> = [];
    try {
      parseResult = await parse(source);
    } catch (error) {
      // It's fine to ignore that.
    }

    // Token decorations.
    for (const { index, 0: image } of source.matchAll(boundaryWordRegex)) {
      if (index !== undefined) {
        const tag = parseResult.findIndex(ids => ids.has(image));
        if (tag !== -1) {
          marks.push([index, index + image.length, tag + 2]);
        }
      }
    }

    // Build decorations.
    marks.sort((x, y) => x[0] - y[0] || x[1] - y[1]);

    const builder = new RangeSetBuilder<Decoration>();
    for (const [start, end, type] of marks) {
      builder.add(start, end, marksPerType[type]);
    }

    return builder.finish();
  }

  return ViewPlugin.fromClass(
    class RegexHighlighter {
      decorations = emptyDecorations;
      updateQueue = Promise.resolve();

      constructor(view: EditorView) {
        this.updateDecorations(view);
      }

      update({ docChanged, view }: ViewUpdate) {
        if (docChanged) {
          this.updateDecorations(view);
        }
      }

      updateDecorations(view: EditorView) {
        // eslint-disable-next-line @typescript-eslint/no-floating-promises -- `getDecorations` never rejects.
        this.updateQueue.then(() =>
          getDecorations(view).then(decorations => {
            this.decorations = decorations;
            view.update([]);
          }),
        );
      }
    },
    { decorations: instance => instance.decorations },
  );
}
