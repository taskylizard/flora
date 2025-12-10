export type MessageContext = {
  msg: {
    id: number;
    channel_id: number;
    guild_id?: number | null;
    content: string;
    author: {
      id: string;
      username: string;
      discriminator?: number | null;
      bot: boolean;
    };
  };
  reply: (content: string) => Promise<void>;
};

export type Command = {
  name: string;
  description?: string;
  run: (ctx: MessageContext & { args: string[] }) => Promise<void> | void;
};

export function defineCommand(command: Command): Command {
  return command;
}

type CreateOptions = {
  prefix?: string;
  commands?: Command[];
  prefixCommands?: Command[];
};

export function createBot(options: CreateOptions) {
  const prefix = options.prefix ?? "!";
  const commands = options.commands ?? options.prefixCommands ?? [];

  on("messageCreate", async (ctx: MessageContext) => {
    if (!ctx.msg || !ctx.msg.content) return;
    if (ctx.msg.author?.bot) return;

    const content = ctx.msg.content.trim();
    if (!content.startsWith(prefix)) return;

    const body = content.slice(prefix.length).trim();
    const [commandName, ...args] = body.split(/\s+/);
    const command = commands.find((cmd) => cmd.name === commandName);
    if (!command) return;

    await command.run({ ...ctx, args });
  });
}
