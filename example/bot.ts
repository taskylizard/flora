const ping = defineCommand({
  name: "ping",
  description: "Respond with pong",
  async run(ctx: MessageContext & { args: string[] }) {
    let content = "Pong!\n"
    content += `Args: ${ctx.args}\n`
    await ctx.reply(content);
  },
});


createBot({
  prefix: ".",
  commands: [ping],
});
