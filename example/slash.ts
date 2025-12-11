const pingSlash = defineSlashCommand({
  name: 'ping',
  description: 'Replies with pong (ephemeral)',
  async run(ctx) {
    await ctx.reply({ content: 'pong', ephemeral: true })
  }
})

const echoSlash = defineSlashCommand({
  name: 'echo',
  description: 'Echo back your input',
  options: [
    {
      name: 'text',
      description: 'What should I repeat?',
      type: 'string',
      required: true,
    },
  ],
  async run(ctx) {
    const content = ctx.msg?.data?.options?.[0]?.value ?? '(nothing)'
    await ctx.reply({ content: `you said: ${content}` })
  }
})

createBot({ slashCommands: [pingSlash, echoSlash] })
