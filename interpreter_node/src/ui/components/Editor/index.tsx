import memoize from 'lodash/memoize';
import * as monaco from 'monaco-editor';
import { createRef, useEffect, useRef } from 'react';
import { initialize as initializeExtenstion } from 'vscode/extensions';
import { LogLevel, initialize as initializeService } from 'vscode/services';

import Client from './client';
import { initialize as initializeLanguage } from './language';
import { createEditor, createModel } from './monaco';
import { initialize as initializeServer } from './server';
import { FromServer, IntoServer } from '../../../codec/codec';
import * as styles from '../../index.module.css';
import { Autosize } from '../Autosize';

const startLSP = memoize(async () => {
  const intoServer: IntoServer = new IntoServer();
  const fromServer: FromServer = FromServer.create();
  const client = new Client(fromServer, intoServer);
  await initializeService({ debugLogging: true, logLevel: LogLevel.Debug });
  await initializeExtenstion();
  initializeLanguage(client);
  initializeServer(intoServer, fromServer).catch(console.error);
  client.start().catch(console.error);
  return client;
});

export type EditorProps = {
  onChange?: (source: string) => void;
  path: string;
  readOnly?: boolean;
  source: string;
};

export function Editor({ onChange, path, readOnly, source }: EditorProps) {
  const editorRef = useRef<monaco.editor.IStandaloneCodeEditor>();
  const ref = createRef<HTMLDivElement>();

  useEffect(() => {
    const div = ref.current;
    if (div === null) {
      return;
    }

    startLSP().then(client => {
      editorRef.current ??= createEditor(client, div, onChange, readOnly);
      editorRef.current?.setModel(createModel(path, source));
    }, console.error);
    return () => editorRef.current?.getModel()?.dispose();
    // eslint-disable-next-line react-hooks/exhaustive-deps -- Updates happen in the following hooks.
  }, [path]);

  useEffect(() => {
    if (readOnly && ref.current && editorRef.current) {
      editorRef.current?.getModel()?.setValue(source);
    }
  }, [readOnly, ref, source]);

  useEffect(() => () => editorRef.current?.dispose(), []);

  return (
    <Autosize>
      {({ height }) => (
        <section className={styles.wrapHidden}>
          <div ref={ref} style={{ height: `${height}px` }} />
        </section>
      )}
    </Autosize>
  );
}
