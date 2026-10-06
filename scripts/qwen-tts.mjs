import { createHash } from "node:crypto";
import { mkdir, writeFile } from "node:fs/promises";
import { resolve, relative, isAbsolute } from "node:path";
import { pathToFileURL } from "node:url";

const model = "qwen-audio-3.1-tts-flash";
const voices = {
  customer: "longanlingxin_v3.1",
  shopkeeper: "xunanchuan_v3.1",
};
export const sample = [
  {
    id: "01",
    role: "customer",
    text: "Bonjour !",
    emotion: "Friendly greeting, gently bright and polite.",
  },
  {
    id: "02",
    role: "shopkeeper",
    text: "Bonjour !",
    emotion: "Warm welcoming reply from a friendly bakery shopkeeper.",
  },
  {
    id: "03",
    role: "customer",
    text: "Je voudrais une baguette, s’il vous plaît.",
    emotion: "A polite request, slightly expectant, relaxed and natural.",
  },
  {
    id: "04",
    role: "shopkeeper",
    text: "Voilà !",
    emotion: "Warm and upbeat, handing over the bread.",
  },
  {
    id: "05",
    role: "customer",
    text: "Merci. C’est combien ?",
    emotion: "Grateful, followed by a natural curious question.",
  },
  {
    id: "06",
    role: "shopkeeper",
    text: "Un euro vingt, s’il vous plaît.",
    emotion: "Matter-of-fact price, warm and courteous.",
  },
  {
    id: "07",
    role: "customer",
    text: "Merci, au revoir !",
    emotion: "A pleased, friendly farewell.",
  },
  {
    id: "08",
    role: "shopkeeper",
    text: "Au revoir !",
    emotion: "Warm farewell with a natural falling cadence.",
  },
];

export function requestFor(line, env) {
  if (
    !env.DASHSCOPE_API_KEY ||
    !/^[a-zA-Z0-9_-]{1,100}$/.test(env.QWEN_WORKSPACE_ID ?? "")
  ) {
    throw new Error(
      "请在 .local/tts.env 配置北京地域 DASHSCOPE_API_KEY 和 QWEN_WORKSPACE_ID。",
    );
  }
  if (
    !line ||
    !/^[a-zA-Z0-9_-]{1,40}$/.test(line.id ?? "") ||
    !voices[line.role] ||
    typeof line.text !== "string" ||
    !line.text.trim() ||
    [...line.text].length > 600 ||
    typeof line.emotion !== "string" ||
    line.emotion.length > 1000
  ) {
    throw new Error("试听台词无效。");
  }
  return {
    endpoint: `https://${env.QWEN_WORKSPACE_ID}.cn-beijing.maas.aliyuncs.com/api/v1/services/audio/tts/SpeechSynthesizer`,
    body: {
      model,
      input: {
        text: line.text,
        voice: voices[line.role],
        format: "wav",
        sample_rate: 24000,
        language_hints: ["fr"],
        rate: 1,
        seed: 0,
        enable_aigc_tag: true,
        instruction: `Speak only the supplied French text, with clear natural French pronunciation for an A1 learner. Do not add words or read these instructions. ${line.emotion}`,
      },
    },
  };
}

// Only the provider's documented result bucket; signed URLs never enter logs or manifests.
export function audioUrl(value) {
  let url;
  try {
    url = new URL(value);
  } catch {
    throw new Error("提供方音频地址无效。");
  }
  if (
    !["http:", "https:"].includes(url.protocol) ||
    url.username ||
    url.password ||
    url.port ||
    url.hostname !== "dashscope-result-bj.oss-cn-beijing.aliyuncs.com"
  ) {
    throw new Error("提供方返回了未允许的音频地址。");
  }
  url.protocol = "https:";
  return url;
}

async function boundedBytes(response, limit) {
  if (!response.ok)
    throw new Error(`语音服务请求失败（HTTP ${response.status}）。`);
  if (!response.body || Number(response.headers.get("content-length")) > limit)
    throw new Error("语音响应超出限额。");
  const reader = response.body.getReader();
  const chunks = [];
  let size = 0;
  try {
    while (true) {
      const { done, value } = await reader.read();
      if (done) break;
      size += value.length;
      if (size > limit) throw new Error("语音响应超出限额。");
      chunks.push(value);
    }
  } finally {
    await reader.cancel().catch(() => {});
  }
  return Buffer.concat(chunks, size);
}

export async function synthesize(line, env, fetcher = fetch) {
  const request = requestFor(line, env);
  let response;
  try {
    response = await fetcher(request.endpoint, {
      method: "POST",
      redirect: "error",
      signal: AbortSignal.timeout(120_000),
      headers: {
        Authorization: `Bearer ${env.DASHSCOPE_API_KEY}`,
        "Content-Type": "application/json",
      },
      body: JSON.stringify(request.body),
    });
  } catch {
    throw new Error("生成连接失败或超时；未自动重试，请先核对提供方计费记录。");
  }
  let result;
  try {
    result = JSON.parse(
      (await boundedBytes(response, 256_000)).toString("utf8"),
    );
  } catch (error) {
    if (error instanceof SyntaxError) throw new Error("提供方响应格式无效。");
    throw error;
  }
  if (result.output?.finish_reason !== "stop" || result.code)
    throw new Error("提供方未返回完成的音频。");
  const url = audioUrl(result.output?.audio?.url);
  let downloaded;
  try {
    downloaded = await fetcher(url, {
      redirect: "error",
      signal: AbortSignal.timeout(60_000),
    });
  } catch {
    throw new Error("生成已完成，但音频下载失败；请核对记录后再重试。");
  }
  const wav = await boundedBytes(downloaded, 16 * 1024 * 1024);
  if (
    wav.length < 44 ||
    wav.toString("ascii", 0, 4) !== "RIFF" ||
    wav.toString("ascii", 8, 12) !== "WAVE"
  ) {
    throw new Error("提供方返回的文件不是 WAV，未保存。");
  }
  return {
    wav,
    parameters: request.body,
    usage: {
      inputTokens: Number.isSafeInteger(result.usage?.input_tokens)
        ? result.usage.input_tokens
        : null,
      outputTokens: Number.isSafeInteger(result.usage?.output_tokens)
        ? result.usage.output_tokens
        : null,
    },
  };
}

export async function run(args, env = process.env) {
  if (args.length !== 1 || !["--plan", "--generate"].includes(args[0]))
    throw new Error("用法：qwen-tts.mjs --plan 或 --generate");
  // Validate credentials before creating any output or issuing paid requests.
  if (args[0] === "--generate") requestFor(sample[0], env);
  const privateRoot = resolve(".local/private/tts-qwen");
  const output = resolve(privateRoot, `bakery-${Date.now()}`);
  const within = relative(privateRoot, output);
  if (within.startsWith("..") || isAbsolute(within))
    throw new Error("输出路径无效。");
  if (args[0] === "--plan")
    return `Qwen Audio 3.1：${sample.length} 句、${sample.reduce((n, line) => n + [...line.text].length, 0)} 个原文字符，2 个支持法语的音色。仅计划，未请求 API。`;
  await mkdir(output, { recursive: true });
  // Per-line receipts survive a later failure. New runs are explicit and bill again.
  for (const line of sample) {
    const result = await synthesize(line, env);
    const receipt = {
      status: "unreviewed",
      provider: "qwen",
      modelVersion: "provider-alias-not-immutable",
      generatedAt: new Date().toISOString(),
      role: line.role,
      id: line.id,
      parameters: result.parameters,
      usage: result.usage,
      inputSha256: createHash("sha256")
        .update(JSON.stringify(result.parameters))
        .digest("hex"),
      audioSha256: createHash("sha256").update(result.wav).digest("hex"),
      bytes: result.wav.length,
    };
    await writeFile(resolve(output, `${line.id}.wav`), result.wav, {
      flag: "wx",
    });
    await writeFile(
      resolve(output, `${line.id}.json`),
      JSON.stringify(receipt, null, 2),
      { flag: "wx" },
    );
  }
  return `试听已保存：${output}。尚未审听、登记或发布。重跑会再次计费。`;
}

if (
  process.argv[1] &&
  import.meta.url === pathToFileURL(resolve(process.argv[1])).href
) {
  try {
    console.log(await run(process.argv.slice(2)));
  } catch (error) {
    // Network and filesystem exceptions can contain signed URLs or environment data.
    const safe =
      /^(请在 |试听台词|提供方|语音服务|语音响应|生成连接|生成已完成|用法：|输出路径)/;
    console.error(
      safe.test(error.message)
        ? error.message
        : "试听生成失败；请核对本机配置与已生成文件，未自动重试。",
    );
    process.exitCode = 1;
  }
}
