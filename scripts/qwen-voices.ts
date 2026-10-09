export * from "../framework/scripts/qwen-voices.ts";
import { forwardCli } from "../framework/scripts/product-cli.ts";
await forwardCli(import.meta.url, new URL("../framework/scripts/qwen-voices.ts", import.meta.url));
