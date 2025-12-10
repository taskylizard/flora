declare global {
  type MessageContext = import("./index").MessageContext;
  type MessageUpdateContext = import("./index").MessageUpdateContext;
  type MessageDeleteContext = import("./index").MessageDeleteContext;
  type MessageDeleteBulkContext = import("./index").MessageDeleteBulkContext;
  type Command = import("./index").Command;

  function on(
    event: "messageCreate",
    handler: (ctx: MessageContext) => void | Promise<void>
  ): void;
  function on(
    event: "messageUpdate",
    handler: (ctx: MessageUpdateContext) => void | Promise<void>
  ): void;
  function on(
    event: "messageDelete",
    handler: (ctx: MessageDeleteContext) => void | Promise<void>
  ): void;
  function on(
    event: "messageDeleteBulk",
    handler: (ctx: MessageDeleteBulkContext) => void | Promise<void>
  ): void;

  const createBot: typeof import("./index").createBot;
  const defineCommand: typeof import("./index").defineCommand;
}

export {};
