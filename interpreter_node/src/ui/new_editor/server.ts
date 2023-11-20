import init, { InitOutput, serve, ServerConfig } from "../../wasm/lsp_module";
import { FromServer, IntoServer } from "./codec";

let server: null | Server;
let initialized = false;

export default class Server {
  readonly #intoServer: IntoServer;
  readonly #fromServer: FromServer;

  constructor(intoServer: IntoServer, fromServer: FromServer) {
    this.#intoServer = intoServer;
    this.#fromServer = fromServer;
  }

  async initialize(): Promise<void> {
    if (null == server) {
      await init();
      server = this;
    } else {
      console.warn("Server already initialized; ignoring");
    }
  }

  async start(): Promise<void> {
    if (initialized) {
      console.warn("Server already started; ignoring");
      return;
    }
    initialized = true;
    const config = new ServerConfig(this.#intoServer, this.#fromServer);
    await serve(config);
  }
}
