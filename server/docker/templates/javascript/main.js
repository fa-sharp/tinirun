import { run } from "./function.js";
import { createInterface } from "node:readline";

const rl = createInterface({ input: process.stdin });

const lines = [];
rl.on("line", (line) => lines.push(line));

rl.on("error", (err) => {
  process.stderr.write(err instanceof Error ? err.message : String(err));
  process.exit(1);
});

rl.on("close", async () => {
  const input = lines.join("\n");
  try {
    const output = await run(input);
    process.stdout.write(output);
  } catch (err) {
    process.stderr.write(err instanceof Error ? err.message : String(err));
    process.exit(1);
  }
});
