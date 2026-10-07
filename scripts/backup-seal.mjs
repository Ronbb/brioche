export * from "../framework/scripts/backup-seal.mjs";
import { forwardCli } from "../framework/scripts/product-cli.mjs";
await forwardCli(import.meta.url, new URL("../framework/scripts/backup-seal.mjs", import.meta.url));
