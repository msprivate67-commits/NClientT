import { defineStore } from "pinia";
import { computed, ref } from "vue";

import {
  apiGetTags,
  tagsAddBlacklist,
  tagsGetAll,
  tagsGetBlacklist,
  tagsRemoveBlacklist,
  tagsSearch,
  tagsSetStatus,
} from "@/api";
import type { Tag, TagStatus, TagType } from "@/types";

export const useTagsStore = defineStore("tags", () => {
  const tags = ref<Tag[]>([]);
  const loaded = ref(false);
  const blacklistLoaded = ref(false);

  const grouped = computed(() => {
    const map = new Map<TagType, Tag[]>();
    for (const t of tags.value) {
      const list = map.get(t.type) ?? [];
      list.push(t);
      map.set(t.type, list);
    }
    return map;
  });

  const accepted = computed(() => tags.value.filter((t) => t.status === "accepted"));
  const avoided = computed(() => tags.value.filter((t) => t.status === "avoided"));
  const blacklisted = computed(() => tags.value.filter((t) => t.blacklisted));
  const blacklistedIds = computed(() => new Set(blacklisted.value.map((t) => t.id)));

  async function load(force = false) {
    if (loaded.value && !force) return tags.value;
    try {
      tags.value = await apiGetTags();
    } catch {
      tags.value = await tagsGetAll();
    }
    loaded.value = true;
    return tags.value;
  }

  async function search(query: string, limit = 50): Promise<Tag[]> {
    if (!query.trim()) {
      await load();
      return tags.value.slice(0, limit);
    }
    const remote = await tagsSearch(query, limit);
    merge(remote);
    return remote;
  }

  async function loadBlacklist(force = false): Promise<Tag[]> {
    if (blacklistLoaded.value && !force) return blacklisted.value;
    const remote = await tagsGetBlacklist();
    for (const tag of tags.value) tag.blacklisted = false;
    merge(remote.map((tag) => ({ ...tag, blacklisted: true })));
    for (const remoteTag of remote) {
      const tag = tags.value.find((candidate) => candidate.id === remoteTag.id);
      if (tag) tag.blacklisted = true;
    }
    blacklistLoaded.value = true;
    return blacklisted.value;
  }

  async function setStatus(id: number, status: TagStatus) {
    await tagsSetStatus(id, status);
    const t = tags.value.find((x) => x.id === id);
    if (t) t.status = status;
  }

  async function cycle(id: number): Promise<TagStatus> {
    const t = tags.value.find((x) => x.id === id);
    const current = t?.status ?? "default";
    const next: TagStatus =
      current === "default" ? "accepted" : current === "accepted" ? "avoided" : "default";
    await setStatus(id, next);
    return next;
  }

  async function addBlacklist(id: number) {
    await tagsAddBlacklist(id);
    const tag = tags.value.find((candidate) => candidate.id === id);
    if (tag) tag.blacklisted = true;
  }
  async function removeBlacklist(id: number) {
    await tagsRemoveBlacklist(id);
    const tag = tags.value.find((candidate) => candidate.id === id);
    if (tag) tag.blacklisted = false;
  }

  function merge(newTags: Tag[]) {
    for (const t of newTags) {
      const existing = tags.value.find((x) => x.id === t.id);
      if (!existing) {
        tags.value.push(t);
      } else {
        existing.name = t.name;
        existing.type = t.type;
        existing.count = t.count;
        if (t.status && t.status !== "default") existing.status = t.status;
        if (t.blacklisted) existing.blacklisted = true;
      }
    }
  }

  return {
    tags,
    loaded,
    grouped,
    accepted,
    avoided,
    blacklisted,
    blacklistedIds,
    load,
    loadBlacklist,
    search,
    setStatus,
    cycle,
    addBlacklist,
    removeBlacklist,
    merge,
  };
});
