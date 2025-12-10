export type EmbedField = {
  name: string;
  value: string;
  inline?: boolean;
};

export type Embed = {
  title?: string;
  description?: string;
  url?: string;
  color?: number;
  footer?: { text: string; iconUrl?: string };
  image?: { url: string };
  thumbnail?: { url: string };
  author?: { name?: string; url?: string; iconUrl?: string };
  fields?: EmbedField[];
};

export type Attachment =
  | { url: string; filename?: string; description?: string }
  | { data: string; filename: string; description?: string };

export type AllowedMentions = {
  parse?: Array<"everyone" | "roles" | "users">;
  users?: string[];
  roles?: string[];
  repliedUser?: boolean;
};

export type MessageReplyOptions = {
  content?: string;
  embeds?: Embed[];
  attachments?: Attachment[];
  tts?: boolean;
  allowedMentions?: AllowedMentions;
  replyTo?: string | null;
};

type BaseContext<TPayload> = {
  msg: TPayload;
  reply: (content: string | MessageReplyOptions) => Promise<void>;
};

export type MessageAuthor = {
  id: string;
  username: string;
  discriminator?: number | null;
  bot: boolean;
};

export type MessagePayload = {
  id: string;
  channel_id: string;
  guild_id?: string | null;
  content: string;
  author: MessageAuthor;
};

export type MessageContext = BaseContext<MessagePayload>;

export type MessageUpdatePayload = {
  id: string;
  channel_id: string;
  guild_id?: string | null;
  content?: string | null;
  author?: MessageAuthor | null;
  edited_timestamp?: string | null;
  old?: MessagePayload | null;
  new?: MessagePayload | null;
};

export type MessageUpdateContext = BaseContext<MessageUpdatePayload>;

export type MessageDeletePayload = {
  id: string;
  channel_id: string;
  guild_id?: string | null;
};

export type MessageDeleteContext = BaseContext<MessageDeletePayload>;

export type MessageDeleteBulkPayload = {
  ids: string[];
  channel_id: string;
  guild_id?: string | null;
};

export type MessageDeleteBulkContext = BaseContext<MessageDeleteBulkPayload>;

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
