export * from "../framework/scripts/qwen-tts.mjs";
import { forwardCli } from "../framework/scripts/product-cli.mjs";
await forwardCli(import.meta.url, new URL("../framework/scripts/qwen-tts.mjs", import.meta.url));
