// Example bot script loaded into the shared runtime using the SDK bundle.
const { createBot, defineCommand } = globalThis.oakmoss || {};

const ping = defineCommand({
  name: "ping",
  description: "Respond with pong",
  async run(ctx: any) {
    await ctx.reply("pong from ts");
  },
});

createBot({
  prefix: "!",
  commands: [ping],
});
