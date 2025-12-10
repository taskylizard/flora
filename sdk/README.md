# oakmoss Runtime & SDK API

Reference documentation for oakmoss.

## Runtime API (global, always available)
- `on(event: string, handler: (ctx) => void | Promise<void>)`: register a handler for a Discord event. Multiple handlers per event are allowed.
- `ctx.msg`: the raw event payload passed from the Rust runtime.
- `ctx.reply(content: string): Promise<void>`: sends a message back to the same channel, automatically replying to the triggering message ID when present.
- `console.log(...args)`: forwarded to Rust tracing (`oakmoss:js`) for structured logs.

### Event names and payload shapes
Events mirror Serenity events bridged in `src/discord_handler.rs`.

```ts
type MessageAuthor = {
  id: string;
  username: string;
  discriminator?: number | null;
  bot: boolean;
};

type MessagePayload = {
  id: string;
  channel_id: string;
  guild_id?: string | null;
  content: string;
  author: MessageAuthor;
};

type MessageUpdatePayload = {
  id: string;
  channel_id: string;
  guild_id?: string | null;
  content?: string | null;
  author?: MessageAuthor | null;
  edited_timestamp?: string | null;
  old?: MessagePayload | null;
  new?: MessagePayload | null;
};

// Events you can subscribe to
on("ready", (ctx) => { /* ctx.msg: { user, guild_ids } */ });
on("messageCreate", (ctx: { msg: MessagePayload; reply: Function }) => { /* … */ });
on("messageUpdate", (ctx: { msg: MessageUpdatePayload; reply: Function }) => { /* … */ });
on("messageDelete", (ctx) => { /* ctx.msg: { id, channel_id, guild_id? } */ });
on("messageDeleteBulk", (ctx) => { /* ctx.msg: { ids: string[], channel_id, guild_id? } */ });
```

Notes:
- Handlers run inside a single-threaded V8 isolate per guild; avoid blocking work.
- `reply` stringifies non-string inputs; use plain text today (embeds/attachments are not yet supported).
- If no guild-specific isolate exists, events fall back to the default runtime.

## SDK API (imported from `dist/sdk-bundle.js`)
The SDK builds on the runtime helpers to simplify prefix-style commands.

```ts
/// defineCommand, createBot are globally available, see example/ dir
const ping = defineCommand({
  name: "ping",
  description: "Respond with pong",
  run: async (ctx) => {
    await ctx.reply(`pong (${ctx.args.join(" ") || "no args"})`);
  },
});

createBot({
  prefix: "!",           // optional; defaults to "!"
  commands: [ping],      // or use prefixCommands for legacy naming
});
```

### Exports
- `defineCommand(command: { name: string; description?: string; run(ctx): void | Promise<void> })`: returns the command unchanged; use it for type safety and clarity.
- `createBot(options)`: wires message handlers for prefix commands.
  - `options.prefix?: string` — command prefix (default `"!"`).
  - `options.commands?: Command[]` — commands to register (preferred).
  - `options.prefixCommands?: Command[]` — alias of `commands` for compatibility.
- Types re-exported for consumers: `MessageAuthor`, `MessagePayload`, `MessageContext`, `MessageUpdatePayload`, `MessageUpdateContext`, `MessageDeletePayload`, `MessageDeleteContext`, `MessageDeleteBulkPayload`, `MessageDeleteBulkContext`, `Command`.

### How command dispatch works
- The SDK registers an internal `on("messageCreate")` handler.
- Incoming messages are ignored if authored by bots or if the content does not start with the configured prefix.
- The first token after the prefix is matched against `command.name`; remaining tokens are passed as `ctx.args`.
- `ctx.reply` routes through the runtime `op_send_message` to Discord with a message reference when possible.

## Development tips
- Type definitions live in `dist/types` for editor intellisense when consuming the bundled SDK.
- Rebuild the bundle after SDK edits: `bun run sdk/build.ts` (run from repo root).
- For custom scripts outside the SDK, rely on the runtime globals (`on`, `console.log`, `ctx.reply`) and keep the event payload shapes above handy.
