import { FromServer, IntoServer } from '../../../codec/codec';
import init, { serve, ServerConfig } from '../../../wasm/lsp';

export const initialize = async (
  intoServer: IntoServer,
  fromServer: FromServer,
) => {
  await init();
  const config = new ServerConfig(intoServer, fromServer);
  await serve(config);
};
