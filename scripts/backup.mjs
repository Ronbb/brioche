export * from "../framework/scripts/backup.mjs";
import { forwardCli } from "../framework/scripts/product-cli.mjs";
await forwardCli(import.meta.url, new URL("../framework/scripts/backup.mjs", import.meta.url));
