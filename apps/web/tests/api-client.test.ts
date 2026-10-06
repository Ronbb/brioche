import test from "node:test";
import assert from "node:assert/strict";
import {
  ApiRequestError,
  definitiveWriteFailure,
  privateRequest,
} from "../app/lib/api.client.ts";

test("CSRF bootstrap rejection never impersonates a mutation rejection", async () => {
  const original = globalThis.fetch;
  try {
    for (const status of [400, 404, 409, 410, 422]) {
      const paths: string[] = [];
      globalThis.fetch = async (input) => {
        paths.push(String(input));
        return new Response(null, { status });
      };
      await assert.rejects(
        privateRequest("/api/v1/learning-sessions/test/attempts", "POST", {
          idempotencyKey: "original",
        }),
        (failure) => {
          assert.ok(failure instanceof ApiRequestError);
          assert.equal(failure.phase, "csrf");
          assert.equal(failure.status, status);
          assert.equal(definitiveWriteFailure(failure), false);
          return true;
        },
      );
      assert.deepEqual(paths, ["/api/v1/auth/csrf"]);
    }
  } finally {
    globalThis.fetch = original;
  }
});

test("only the target response can conclusively reject the exact mutation", async () => {
  const original = globalThis.fetch;
  try {
    const calls: { path: string; init?: RequestInit }[] = [];
    const body = {
      version: 4,
      idempotencyKey: "exact-original",
      answer: { kind: "text", text: "une" },
    };
    globalThis.fetch = async (input, init) => {
      calls.push({ path: String(input), init });
      return calls.length === 1
        ? Response.json({ csrfToken: "test-csrf" })
        : new Response(null, { status: 410 });
    };
    await assert.rejects(
      privateRequest("/api/v1/learning-sessions/test/attempts", "POST", body),
      (failure) => {
        assert.ok(failure instanceof ApiRequestError);
        assert.equal(failure.phase, "request");
        assert.equal(definitiveWriteFailure(failure), true);
        return true;
      },
    );
    assert.equal(calls.length, 2);
    assert.equal(calls[1].path, "/api/v1/learning-sessions/test/attempts");
    assert.equal(calls[1].init?.method, "POST");
    assert.equal(calls[1].init?.body, JSON.stringify(body));
    assert.equal(
      new Headers(calls[1].init?.headers).get("X-CSRF-Token"),
      "test-csrf",
    );
  } finally {
    globalThis.fetch = original;
  }
});
