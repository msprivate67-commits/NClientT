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

function requestHeaders(apiKey: string): Record<string, string> {
  const headers: Record<string, string> = { "Content-Type": "application/json" };
  if (apiKey) headers.Authorization = `Bearer ${apiKey}`;
  return headers;
}

export async function translateTitle(
  baseUrl: string,
  model: string,
  apiKey: string,
  title: string,
  targetLang: string,
  thinking: boolean,
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

  const response = await fetch(endpoint(baseUrl), {
    method: "POST",
    headers: requestHeaders(apiKey),
    body: JSON.stringify(body),
    signal: handlers.signal,
  });
  if (!response.ok) {
    throw new Error(`Translation API error (${response.status}): ${await response.text()}`);
  }

  const contentType = response.headers.get("Content-Type")?.toLowerCase() ?? "";
  if (!response.body || contentType.includes("application/json")) {
    const payload: unknown = await response.json();
    const message = extractMessage(payload);
    if (message.reasoning) handlers.onReasoning?.(message.reasoning);
    if (message.content) handlers.onContent?.(message.content);
    if (!message.content.trim()) throw new Error("Empty translation response");
    return message.content.trim();
  }

  const reader = response.body.getReader();
  const decoder = new TextDecoder();
  let buffer = "";
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

  while (true) {
    const { done, value } = await reader.read();
    buffer += decoder.decode(value, { stream: !done });
    const lines = buffer.split(/\r?\n/);
    buffer = lines.pop() ?? "";
    lines.forEach(consumeLine);
    if (done) break;
  }
  if (buffer.trim()) consumeLine(buffer);

  if (!content.trim()) throw new Error("Empty translation response");
  return content.trim();
}

export async function testTranslationConnection(
  baseUrl: string,
  model: string,
  apiKey: string,
): Promise<TranslationConnectionResult> {
  if (!baseUrl.trim()) return { ok: false, message: "Base URL is empty" };
  if (!model.trim()) return { ok: false, message: "Model is empty" };

  const controller = new AbortController();
  const timeout = setTimeout(() => controller.abort(), 10_000);
  try {
    const response = await fetch(endpoint(baseUrl), {
      method: "POST",
      headers: requestHeaders(apiKey),
      body: JSON.stringify({
        model,
        messages: [{ role: "user", content: "ping" }],
        max_tokens: 1,
      }),
      signal: controller.signal,
    });
    if (response.ok) return { ok: true, message: "" };

    let detail = "";
    try {
      detail = (await response.text()).slice(0, 200);
    } catch {
      // The status code is still useful when the provider closes the body.
    }
    return { ok: false, message: `HTTP ${response.status}${detail ? ` — ${detail}` : ""}` };
  } catch (error: unknown) {
    if (error instanceof DOMException && error.name === "AbortError") {
      return { ok: false, message: "Timed out" };
    }
    return { ok: false, message: error instanceof Error ? error.message : String(error) };
  } finally {
    clearTimeout(timeout);
  }
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
