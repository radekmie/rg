import memoize from 'lodash/memoize';
import { editor } from 'monaco-editor';
import { createRef, useEffect, useRef } from 'react';
import { initialize as initializeExtenstion } from 'vscode/extensions';
import { LogLevel, initialize as initializeService } from 'vscode/services';

import Client from './client';
import { FromServer, IntoServer } from './codec';
import { initialize as initializeLanguage } from './language';
import { createEditor, createModel } from './monaco_editor';
import Server from './server';
import { Autosize } from '../components/Autosize';
import * as styles from '../index.module.css';

const startLSP = memoize(async () => {
  const intoServer: IntoServer = new IntoServer();
  const fromServer: FromServer = FromServer.create();
  const client = new Client(fromServer, intoServer);
  const server = new Server(intoServer, fromServer);
  await initializeService({ debugLogging: true, logLevel: LogLevel.Debug });
  await initializeExtenstion();
  await server.initialize();
  initializeLanguage(client);
  client.start();
  server.start();
  return client;
});

export type EditorProps = {
  path: string;
  source: string;
  onChange: (source: string) => void;
  readonly: boolean;
  className?: string;
};

export function Editor({
  path,
  source,
  onChange,
  readonly,
  className,
}: EditorProps) {
  const editorRef = useRef<editor.IStandaloneCodeEditor>();
  const ref = createRef<HTMLDivElement>();
  useEffect(() => {
    if (ref.current !== null) {
      const start = async () => {
        // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
        const crr = ref.current!;
        const client = await startLSP();
        if (!editorRef.current) {
          editorRef.current = createEditor(client, crr, onChange, readonly);
        }
        const model = createModel(path, source);
        editorRef.current?.setModel(model);
      };
      start();
      return () => {
        const currentEditor = editorRef.current;
        const model = currentEditor?.getModel();
        if (model) {
          model.dispose();
        }
      };
    }
  }, [path]);

  useEffect(() => {
    if (readonly && ref.current && editorRef.current) {
      const model = editorRef.current?.getModel();
      if (model) {
        model.setValue(source);
      }
    }
  }, [readonly, ref, source]);

  useEffect(() => {
    return () => {
      const currentEditor = editorRef.current;
      if (currentEditor) {
        currentEditor.dispose();
      }
    };
  }, []);

  return (
    <Autosize>
      {({ height }) => (
        <section className={styles.wrapHidden}>
          <div
            ref={ref}
            style={{ height: `${height}px` }}
            className={className}
          />
        </section>
      )}
    </Autosize>
  );
}
