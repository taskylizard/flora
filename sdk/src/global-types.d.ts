declare global {
  type MessageContext = import("./index").MessageContext;
  type Command = import("./index").Command;

  function on(
    event: "messageCreate",
    handler: (ctx: MessageContext) => void | Promise<void>
  ): void;

  const createBot: typeof import("./index").createBot;
  const defineCommand: typeof import("./index").defineCommand;
}

export {};
