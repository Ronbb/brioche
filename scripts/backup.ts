export * from "../framework/scripts/backup.ts";
import { forwardCli } from "../framework/scripts/product-cli.ts";
await forwardCli(import.meta.url, new URL("../framework/scripts/backup.ts", import.meta.url));
