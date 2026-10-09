export * from "../framework/scripts/health-check.ts";
import { forwardCli } from "../framework/scripts/product-cli.ts";
await forwardCli(import.meta.url, new URL("../framework/scripts/health-check.ts", import.meta.url));
