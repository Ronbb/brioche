import test from "node:test";
import assert from "node:assert/strict";
import { adminWrite } from "../app/lib/admin.client.ts";
test("canceled admin writes cannot issue a token after late CSRF bootstrap", async () => {
  const original = globalThis.fetch;
  const controller = new AbortController();
  const paths: string[] = [];
  try {
    globalThis.fetch = async (input) => {
      paths.push(String(input));
      return {
        ok: true,
        json: async () => {
          controller.abort();
          return { csrfToken: "obsolete" };
        },
      } as Response;
    };
    await assert.rejects(
      adminWrite(
        "accounts/token",
        { email: "synthetic@example.test" },
        controller.signal,
      ),
      { name: "AbortError" },
    );
    assert.deepEqual(paths, ["/api/v1/auth/csrf"]);
    await assert.rejects(adminWrite("accounts/token", {}, controller.signal), {
      name: "AbortError",
    });
    assert.equal(paths.length, 1);
  } finally {
    globalThis.fetch = original;
  }
});
