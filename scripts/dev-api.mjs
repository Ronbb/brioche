import { spawn } from "node:child_process";
const child = spawn("cargo", ["run", "-p", "brioche-server"], {
  stdio: "inherit",
  env: {
    ...process.env,
    APP_ENV: process.env.APP_ENV ?? "development",
    CONTENT_MODE: process.env.CONTENT_MODE ?? "fixture",
  },
});
for (const signal of ["SIGINT", "SIGTERM"])
  process.on(signal, () => child.kill(signal));
child.on("exit", (code) => process.exit(code ?? 1));
