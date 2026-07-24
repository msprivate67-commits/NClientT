import { Channel, invoke } from "@tauri-apps/api/core";

export interface TranslationConnectionResult {
  ok: boolean;
  message: string;
}

export interface TranslationStreamHandlers {
  onContent?: (chunk: string) => void;
  onReasoning?: (chunk: string) => void;
  signal?: AbortSignal;
}

function endpoint(baseUrl: string): string {
  return `${baseUrl.replace(/\/+$/, "")}/v1/chat/completions`;
}

export async function translateTitle(
  baseUrl: string,
  model: string,
  apiKey: string,
  title: string,
  targetLang: string,
  thinking: boolean,
  useProxy: boolean,
  handlers: TranslationStreamHandlers = {},
): Promise<string> {
  const body: Record<string, unknown> = {
    model,
    messages: [
      {
        role: "system",
        content: `You are a translator. The following text is a hentai manga title. The original language is either Japanese or English. Translate it to ${targetLang}. You must use ${targetLang} for both your reasoning process and your final output. Output ONLY the translated title in the final answer, with no quotes, extra words, or explanations.`,
      },
      { role: "user", content: title },
    ],
    stream: true,
  };
  applyThinkingControl(body, baseUrl, model, thinking);
  if (!thinking) body.temperature = 0.1;

  const collector = createStreamCollector(handlers);
  const onChunk = new Channel<number[]>();
  onChunk.onmessage = (chunk) => {
    if (!handlers.signal?.aborted) collector.push(new Uint8Array(chunk));
  };
  await invokeWithAbort(invoke("translation_stream_request", {
    url: endpoint(baseUrl),
    apiKey,
    body,
    useProxy,
    onChunk,
  }), handlers.signal);
  return collector.finish();
}

export async function testTranslationConnection(
  baseUrl: string,
  model: string,
  apiKey: string,
  useProxy: boolean,
): Promise<TranslationConnectionResult> {
  if (!baseUrl.trim()) return { ok: false, message: "Base URL is empty" };
  if (!model.trim()) return { ok: false, message: "Model is empty" };

  try {
    const body = {
      model,
      messages: [{ role: "user", content: "ping" }],
      max_tokens: 1,
    };
    const onChunk = new Channel<number[]>();
    onChunk.onmessage = () => {};
    await withTimeout(invoke("translation_stream_request", {
      url: endpoint(baseUrl),
      apiKey,
      body,
      useProxy,
      onChunk,
    }), 10_000);
    return { ok: true, message: "" };
  } catch (error: unknown) {
    return { ok: false, message: error instanceof Error ? error.message : String(error) };
  }
}

async function invokeWithAbort<T>(request: Promise<T>, signal?: AbortSignal): Promise<T> {
  if (!signal) return request;
  if (signal.aborted) throw new DOMException("Aborted", "AbortError");
  return Promise.race([
    request,
    new Promise<T>((_, reject) => {
      signal.addEventListener(
        "abort",
        () => reject(new DOMException("Aborted", "AbortError")),
        { once: true },
      );
    }),
  ]);
}

async function withTimeout<T>(request: Promise<T>, timeoutMs: number): Promise<T> {
  let timeout: ReturnType<typeof setTimeout> | null = null;
  try {
    return await Promise.race([
      request,
      new Promise<T>((_, reject) => {
        timeout = setTimeout(() => reject(new Error("Timed out")), timeoutMs);
      }),
    ]);
  } finally {
    if (timeout) clearTimeout(timeout);
  }
}

function createStreamCollector(handlers: TranslationStreamHandlers) {
  const decoder = new TextDecoder();
  let buffer = "";
  let raw = "";
  let content = "";

  const consumeLine = (line: string) => {
    const trimmed = line.trim();
    if (!trimmed || trimmed.startsWith(":")) return;
    const data = trimmed.startsWith("data:") ? trimmed.slice(5).trim() : trimmed;
    if (!data || data === "[DONE]") return;
    try {
      const delta = extractDelta(JSON.parse(data) as unknown);
      if (delta.reasoning) handlers.onReasoning?.(delta.reasoning);
      if (delta.content) {
        content += delta.content;
        handlers.onContent?.(delta.content);
      }
    } catch {
      // Ignore non-JSON SSE metadata emitted by some compatible providers.
    }
  };

  const append = (text: string) => {
    raw += text;
    buffer += text;
    const lines = buffer.split(/\r?\n/);
    buffer = lines.pop() ?? "";
    lines.forEach(consumeLine);
  };

  return {
    push(bytes: Uint8Array) {
      append(decoder.decode(bytes, { stream: true }));
    },
    finish(): string {
      append(decoder.decode());
      if (buffer.trim()) consumeLine(buffer);
      if (content.trim()) return content.trim();

      try {
        const message = extractMessage(JSON.parse(raw) as unknown);
        if (message.reasoning) handlers.onReasoning?.(message.reasoning);
        if (message.content) handlers.onContent?.(message.content);
        if (message.content.trim()) return message.content.trim();
      } catch {
        // The empty-response error below is clearer than a JSON parse error.
      }
      throw new Error("Empty translation response");
    },
  };
}

function extractDelta(payload: unknown): { content: string; reasoning: string } {
  if (!payload || typeof payload !== "object") return { content: "", reasoning: "" };
  const choices = (payload as { choices?: unknown }).choices;
  if (!Array.isArray(choices)) return { content: "", reasoning: "" };
  const first = choices[0];
  if (!first || typeof first !== "object") return { content: "", reasoning: "" };
  const delta = (first as { delta?: unknown }).delta;
  if (!delta || typeof delta !== "object") return { content: "", reasoning: "" };
  return extractParts(delta);
}

function extractMessage(payload: unknown): { content: string; reasoning: string } {
  if (!payload || typeof payload !== "object") return { content: "", reasoning: "" };
  const choices = (payload as { choices?: unknown }).choices;
  if (!Array.isArray(choices)) return { content: "", reasoning: "" };
  const first = choices[0];
  if (!first || typeof first !== "object") return { content: "", reasoning: "" };
  const message = (first as { message?: unknown }).message;
  return message && typeof message === "object"
    ? extractParts(message)
    : { content: "", reasoning: "" };
}

function extractParts(value: object): { content: string; reasoning: string } {
  const part = value as {
    content?: unknown;
    reasoning?: unknown;
    reasoning_content?: unknown;
  };
  return {
    content: typeof part.content === "string" ? part.content : "",
    reasoning: typeof part.reasoning_content === "string"
      ? part.reasoning_content
      : typeof part.reasoning === "string" ? part.reasoning : "",
  };
}

function applyThinkingControl(
  body: Record<string, unknown>,
  baseUrl: string,
  model: string,
  enabled: boolean,
) {
  const base = baseUrl.toLowerCase();
  const modelName = model.toLowerCase();

  if (base.includes("deepseek") || modelName.startsWith("deepseek-")) {
    // DeepSeek V4 defaults to thinking mode, so the switch must be sent in
    // both directions rather than omitted when disabled.
    body.thinking = { type: enabled ? "enabled" : "disabled" };
    if (enabled) body.reasoning_effort = "high";
    return;
  }

  if (
    base.includes("dashscope")
    || base.includes("aliyuncs")
    || modelName.startsWith("qwen")
    || modelName.startsWith("qwq")
  ) {
    // Qwen's OpenAI-compatible Chat Completions API uses this top-level,
    // provider-specific boolean for hybrid-thinking models.
    body.enable_thinking = enabled;
    return;
  }

  if (base.includes("openrouter.ai")) {
    // OpenRouter normalizes provider-specific controls behind one object.
    body.reasoning = { effort: enabled ? "high" : "none" };
    return;
  }

  const isOpenAiReasoningModel = /^(gpt-5|o1|o3|o4)(?:[-.:]|$)/.test(modelName);
  if (base.includes("api.openai.com") && !isOpenAiReasoningModel) {
    // Non-reasoning OpenAI models do not accept reasoning_effort and already
    // behave like thinking is disabled.
    return;
  }

  if (
    base.includes("generativelanguage.googleapis.com")
    || (base.includes("api.openai.com") && isOpenAiReasoningModel)
    || base.includes("localhost:11434")
    || base.includes("127.0.0.1:11434")
    || base.includes("ollama")
  ) {
    // OpenAI, Gemini's compatibility endpoint, and Ollama understand the
    // OpenAI-style effort field. Individual reasoning-only models may reject
    // "none" because their thinking cannot be disabled.
    body.reasoning_effort = enabled ? "high" : "none";
    return;
  }

  // For unknown compatible providers, use only the commonly accepted enable
  // field. When disabled, omit private parameters: sending every vendor's
  // switch at once would make strict OpenAI-compatible servers reject the
  // otherwise valid request.
  if (enabled) body.reasoning_effort = "high";
}
