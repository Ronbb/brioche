export * from "../framework/scripts/health-check.mjs";
import { forwardCli } from "../framework/scripts/product-cli.mjs";
await forwardCli(import.meta.url, new URL("../framework/scripts/health-check.mjs", import.meta.url));
