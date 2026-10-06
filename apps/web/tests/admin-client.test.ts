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

test("multipart admin writes preserve browser boundaries and the CSRF header", async () => {
  const original = globalThis.fetch;
  const body = new FormData();
  body.set("document", "{}");
  body.set("file", new Blob(["svg"]), "test.svg");
  try {
    let writes = 0;
    globalThis.fetch = async (input, init) => {
      if (String(input).endsWith("/csrf"))
        return new Response(JSON.stringify({ csrfToken: "controlled" }));
      writes++;
      assert.equal(init?.body, body);
      const headers = new Headers(init?.headers);
      assert.equal(headers.has("content-type"), false);
      assert.equal(headers.get("x-csrf-token"), "controlled");
      return new Response(
        JSON.stringify({ assetId: "test-image", revision: 1 }),
      );
    };
    assert.deepEqual(await adminWrite("assets", body), {
      assetId: "test-image",
      revision: 1,
    });
    assert.equal(writes, 1);
  } finally {
    globalThis.fetch = original;
  }
});
