// Downloaded gallery folders are named `[id] title` (see
// `Gallery#download_folder_name`), so a scanned local library title often starts
// with a `[123456] ` id prefix. Strip it for display — the gallery id is already
// shown separately on the detail page, and the list never needs it.
const LEADING_ID_RE = /^\[\s*\d+\s*\]\s*/;

export function stripLeadingId(title: string): string {
  if (!title) return title;
  const stripped = title.replace(LEADING_ID_RE, "");
  // If stripping would leave nothing (folder was just "[123456]"), keep the
  // original so we don't show a blank title.
  return stripped.length > 0 ? stripped : title;
}
