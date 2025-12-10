
const ping = defineCommand({
  name: "ping",
  description: "Respond with pong",
  async run(ctx) {
    const embed: Embed = {
      title: "Pong"
      , fields: [{
        name: "Args",
        value: ctx.args.join(",")
      }]
    }
    await ctx.reply({
      embeds: [embed]
    });
  },
});


createBot({
  prefix: ".",
  commands: [ping],
});
