export interface TranslationConnectionResult {
  ok: boolean;
  message: string;
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
): Promise<string> {
  const body: Record<string, unknown> = {
    model,
    messages: [
      {
        role: "system",
        content: `You are a translator. The following text is a hentai manga title. The original language is either Japanese or English. Translate it to ${targetLang}. Output ONLY the translated title, nothing else — no quotes, no extra words, no explanations.`,
      },
      { role: "user", content: title },
    ],
    temperature: 0.1,
  };
  if (thinking) body.reasoning_effort = "medium";

  const response = await fetch(endpoint(baseUrl), {
    method: "POST",
    headers: requestHeaders(apiKey),
    body: JSON.stringify(body),
  });
  if (!response.ok) {
    throw new Error(`Translation API error (${response.status}): ${await response.text()}`);
  }

  const payload: unknown = await response.json();
  const content = extractMessageContent(payload);
  if (!content) throw new Error("Empty translation response");
  return content;
}

export async function testTranslationConnection(
  baseUrl: string,
  model: string,
  apiKey: string,
): Promise<TranslationConnectionResult> {
  if (!baseUrl.trim()) return { ok: false, message: "Base URL is empty" };
  if (!model.trim()) return { ok: false, message: "Model is empty" };

  try {
    const response = await fetch(endpoint(baseUrl), {
      method: "POST",
      headers: requestHeaders(apiKey),
      body: JSON.stringify({
        model,
        messages: [{ role: "user", content: "ping" }],
        max_tokens: 1,
      }),
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
  }
}

function extractMessageContent(payload: unknown): string {
  if (!payload || typeof payload !== "object") return "";
  const choices = (payload as { choices?: unknown }).choices;
  if (!Array.isArray(choices)) return "";
  const first = choices[0];
  if (!first || typeof first !== "object") return "";
  const message = (first as { message?: unknown }).message;
  if (!message || typeof message !== "object") return "";
  const content = (message as { content?: unknown }).content;
  return typeof content === "string" ? content.trim() : "";
}
