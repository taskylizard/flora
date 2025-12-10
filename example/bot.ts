const ping = defineCommand({
  name: "ping",
  description: "Respond with pong",
  async run(ctx: MessageContext & { args: string[] }) {
    await ctx.reply("pong from example");
  },
});

on("messageCreate", (ctx) => {
  console.log(ctx)
})
createBot({
  prefix: "!",
  commands: [ping],
});
