import { createRef, useEffect, useRef } from 'react';
import { editor } from 'monaco-editor';
import Client from "./client";
import { FromServer, IntoServer } from "./codec";
import Server from "./server";
import MonacoEditor from './monaco_editor';
import { LogLevel, initialize as initializeService } from 'vscode/services';
import { initialize as initializeExtenstion } from 'vscode/extensions';
import { Autosize } from '../components/Autosize';


let init = true;

export type EditorProps = {
  path: string;
  source: string;
  onChange: (source: string) => void;
  className?: string;
}


export function ReactMonacoEditor({
  path,
  source,
  onChange,
  className
}: EditorProps) {

  const intoServer: IntoServer = new IntoServer();
  const fromServer: FromServer = FromServer.create();
  const client = new Client(fromServer, intoServer);
  const server = new Server(intoServer, fromServer);

  const editorRef = useRef<editor.IStandaloneCodeEditor>();
  const ref = createRef<HTMLDivElement>();

  useEffect(() => {
    if (ref.current != null) {
      const start = async () => {
        const crr = ref.current!; 
        if (init) {
          init = false;
          await initializeService({
            debugLogging: true,
            logLevel: LogLevel.Debug       
          });
          await initializeExtenstion();
          await server.initialize();
          Promise.all([server.start(), client.start()]);
        }
        editorRef.current = await new MonacoEditor().createEditor(
          client,
          crr,
          path,
          source,
          onChange
        );
      };
      start();

      return () => {
        const currentEditor = editorRef.current;
        const model = currentEditor?.getModel();
        if (model) {
          model.dispose();
        }
        currentEditor?.dispose();
      };
    }

  }, [path]);

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
      style={{ height: `100vh` }}
      className={className}
    />
  );
};