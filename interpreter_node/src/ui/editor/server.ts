import { FromServer, IntoServer } from '../../codec/codec';
import init, { serve, ServerConfig } from '../../wasm/lsp_module';

let initialized = false;

export default class Server {
  readonly #intoServer: IntoServer;
  readonly #fromServer: FromServer;

  constructor(intoServer: IntoServer, fromServer: FromServer) {
    this.#intoServer = intoServer;
    this.#fromServer = fromServer;
  }

  async initialize(): Promise<void> {
    await init();
  }

  async start(): Promise<void> {
    if (initialized) {
      console.warn('Server already started; ignoring');
      return;
    }
    initialized = true;
    const config = new ServerConfig(this.#intoServer, this.#fromServer);
    await serve(config);
  }
}
