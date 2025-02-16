// This file is copied from https://github.com/silvanshade/tower-lsp-web-demo

import * as jsrpc from 'json-rpc-2.0';
import * as proto from 'vscode-languageserver-protocol';

import { Codec, FromServer, IntoServer } from '../../../codec/codec';

export default class Client extends jsrpc.JSONRPCServerAndClient {
  afterInitializedHooks: (() => Promise<void>)[] = [];
  #fromServer: FromServer;

  constructor(fromServer: FromServer, intoServer: IntoServer) {
    super(
      new jsrpc.JSONRPCServer(),
      new jsrpc.JSONRPCClient(async (json: jsrpc.JSONRPCRequest) => {
        const encoded = Codec.encode(json);
        intoServer.enqueue(encoded);
        if (json.id !== undefined && json.id !== null) {
          const response = await fromServer.responses.get(json.id)!;
          this.client.receive(response as jsrpc.JSONRPCResponse);
        }
      }),
    );
    this.#fromServer = fromServer;
  }

  async start(): Promise<void> {
    // process 'window/logMessage': client <- server
    this.addMethod(proto.LogMessageNotification.type.method, params => {
      const { message, type } = params as {
        message: string;
        type: proto.MessageType;
      };

      switch (type) {
        case proto.MessageType.Error: {
          console.error(message);
          break;
        }
        case proto.MessageType.Warning: {
          console.warn(message);
          break;
        }
        case proto.MessageType.Info: {
          console.log(message);
          break;
        }
        case proto.MessageType.Log: {
          console.debug(message);
          break;
        }
      }
    });

    // request 'initialize': client <-> server
    await this.request(proto.InitializeRequest.type.method, {
      capabilities: {},
      clientInfo: { name: 'demo-language-client' },
      processId: null,
      rootUri: null,
    });

    // notify 'initialized': client --> server
    this.notify(proto.InitializedNotification.type.method, {});

    await Promise.all(this.afterInitializedHooks.map(fn => fn()));
    await Promise.all([this.processNotifications(), this.processRequests()]);
  }

  async processNotifications() {
    for await (const notification of this.#fromServer.notifications) {
      await this.receiveAndSend(notification);
    }
  }

  async processRequests() {
    for await (const request of this.#fromServer.requests) {
      await this.receiveAndSend(request);
    }
  }

  pushAfterInitializeHook(...hooks: (() => Promise<void>)[]) {
    this.afterInitializedHooks.push(...hooks);
  }
}
