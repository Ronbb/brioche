export * from "../framework/scripts/qwen-tts.ts";
import { forwardCli } from "../framework/scripts/product-cli.ts";
await forwardCli(import.meta.url, new URL("../framework/scripts/qwen-tts.ts", import.meta.url));
