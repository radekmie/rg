import React, { createRef, useEffect, useMemo, useRef } from 'react';
import { Uri, editor } from 'monaco-editor';
import Client from "./client";
import { FromServer, IntoServer } from "./codec";
import Language from "./language";
import Server from "./server";
import MonacoEditor from './monaco_editor';
import { initialize } from 'vscode/extensions';

let init = true;

export type EditorProps = {
  defaultCode: string;
  path: string;
  className?: string;
}


export const ReactMonacoEditor: React.FC<EditorProps> = ({
  defaultCode,
  path,
  className
}) => {



  const editorRef = useRef<editor.IStandaloneCodeEditor>();
  const ref = createRef<HTMLDivElement>();

  useEffect(() => {
    const currentEditor = editorRef.current;
    const intoServer: IntoServer = new IntoServer();
    const fromServer: FromServer = FromServer.create();

    if (ref.current != null) {
      const start = async () => {
        const client = new Client(fromServer, intoServer);
        const server = await Server.initialize(intoServer, fromServer);
        await initialize();
        await new MonacoEditor().createEditor(
          client,
          ref.current!,
          'abc'
        );
        await Promise.all([server.start(), client.start()]);
        if (init) {
          init = false;
        }
      };
      start();

      return () => {
        currentEditor?.dispose();
      };
    }

    window.onbeforeunload = () => {
      // On page reload/exit, close web socket connection
    };
    return () => {
      // On component unmount, close web socket connection
    };
  }, []);

  return (
    <div
      ref={ref}
      style={{ height: '50vh' }}
      className={className}
    />
  );
};