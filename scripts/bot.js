// Example bot script loaded into the shared runtime using the SDK bundle.
const { createBot, defineCommand } = globalThis.oakmoss || {};

const ping = defineCommand({
  name: "ping",
  description: "Respond with pong",
  async run(ctx) {
    await ctx.reply("pong");
  },
});

createBot({
  prefix: "!",
  commands: [ping],
});
