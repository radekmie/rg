import { javascript } from '@codemirror/lang-javascript';
import { json } from '@codemirror/lang-json';
import CodeMirror, { ReactCodeMirrorProps } from '@uiw/react-codemirror';

import { Autosize } from './Autosize';

const modeToConfig = {
  javascript: { extensions: [javascript()] },
  json: { extensions: [json()] },
  text: { extensions: [] },
};

export type EditorProps = ReactCodeMirrorProps & {
  mode: keyof typeof modeToConfig;
};

export function Editor({ className, mode, ...props }: EditorProps) {
  return (
    <Autosize>
      {({ height }) => (
        <section className={className}>
          <CodeMirror
            height={`${height}px`}
            {...modeToConfig[mode]}
            {...props}
          />
        </section>
      )}
    </Autosize>
  );
}
