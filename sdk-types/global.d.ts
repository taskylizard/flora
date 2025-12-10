declare global {
  type MessageContext = import("sdk").MessageContext;
  type Command = import("sdk").Command;

  function on(
    event: "messageCreate",
    handler: (ctx: MessageContext) => void | Promise<void>
  ): void;

  const createBot: typeof import("sdk").createBot;
  const defineCommand: typeof import("sdk").defineCommand;
}

export {};
