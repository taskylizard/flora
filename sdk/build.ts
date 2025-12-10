import { rolldown } from "rolldown";
import { writeFileSync } from "node:fs";
import { resolve } from "node:path";

const entry = resolve(import.meta.dirname, "src/index.ts");
const output = resolve(import.meta.dirname, "../scripts/sdk-bundle.js");

const bundle = await rolldown({
  input: entry,
  platform: "neutral",
  treeshake: false,
});

const { output: chunks } = await bundle.generate({
  format: "iife",
  name: "oakmoss",
  exports: "named",
});

const chunk = chunks.find((item) => item.type === "chunk");
if (!chunk || chunk.type !== "chunk") {
  throw new Error("rolldown did not emit a chunk");
}

let code = chunk.code;
code += `
;(function (global) {
  if (!global.oakmoss) return;
  global.createBot = global.oakmoss.createBot;
  global.defineCommand = global.oakmoss.defineCommand;
})(globalThis);
`;

writeFileSync(output, code, "utf8");
console.log(`sdk bundle written to ${output}`);
