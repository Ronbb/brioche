export * from "../framework/scripts/qwen-voices.mjs";
import { forwardCli } from "../framework/scripts/product-cli.mjs";
await forwardCli(import.meta.url, new URL("../framework/scripts/qwen-voices.mjs", import.meta.url));
