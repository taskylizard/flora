var oakmoss = (function(exports) {


//#region src/index.ts
	function defineCommand(command) {
		return command;
	}
	function createBot(options) {
		const prefix = options.prefix ?? "!";
		const commands = options.commands ?? options.prefixCommands ?? [];
		on("messageCreate", async (ctx) => {
			if (!ctx.msg || !ctx.msg.content) return;
			if (ctx.msg.author?.bot) return;
			const content = ctx.msg.content.trim();
			if (!content.startsWith(prefix)) return;
			const body = content.slice(prefix.length).trim();
			const [commandName, ...args] = body.split(/\s+/);
			const command = commands.find((cmd) => cmd.name === commandName);
			if (!command) return;
			await command.run({
				...ctx,
				args
			});
		});
	}

//#endregion
exports.createBot = createBot;
exports.defineCommand = defineCommand;
return exports;
})({});
;(function (global) {
  if (!global.oakmoss) return;
  global.createBot = global.oakmoss.createBot;
  global.defineCommand = global.oakmoss.defineCommand;
})(globalThis);
