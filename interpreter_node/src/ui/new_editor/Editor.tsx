import React, { createRef, useEffect, useMemo, useRef } from 'react';
import { Uri, editor } from 'monaco-editor';
import Client from "./client";
import { FromServer, IntoServer } from "./codec";
import Language from "./language";
import Server from "./server";
import MonacoEditor from './monaco_editor';
import { LogLevel, initialize as initializeService } from 'vscode/services';
import { initialize as initializeExtenstion } from 'vscode/extensions';
import * as fsp from 'fs/promises'
import * as styles from '../index.module.css';
import { Autosize } from '../components/Autosize';
import {presets} from '../const/presets';


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
        const crr = ref.current!;
        const client = new Client(fromServer, intoServer);
        const server = await Server.initialize(intoServer, fromServer);
        if (init) {
          init = false;
          await initializeService({
            debugLogging: true,
            logLevel: LogLevel.Debug
          });
          await initializeExtenstion();
        }
        const preset = presets.find(game => game.name == 'Breakthrough.rg')!.source;
        await new MonacoEditor().createEditor(
          client,
          crr,
          preset,
        );
        await Promise.all([server.start(), client.start()]);
        
      };
      start();

      return () => {
        currentEditor?.dispose();
      };
    }

  }, []);

  return (
    // <Autosize>
    //   {({ height }) => (
    //     <section className={styles.wrapHidden}>
    //       <div
    //         ref={ref}
    //         style={{height: `${height}px`}}
    //         className={className}
    //       />
    //     </section>
    //   )}
    // </Autosize>
    <div
      ref={ref}
      style={{height: `100vh`}}
      className={className}
    />
  );
};