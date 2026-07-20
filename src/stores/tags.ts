import { defineStore } from "pinia";
import { computed, ref } from "vue";

import {
  apiGetTags,
  tagsAddBlacklist,
  tagsGetAll,
  tagsRemoveBlacklist,
  tagsSearch,
  tagsSetStatus,
} from "@/api";
import type { Tag, TagStatus, TagType } from "@/types";

export const useTagsStore = defineStore("tags", () => {
  const tags = ref<Tag[]>([]);
  const loaded = ref(false);

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
  const blacklisted = computed(() => tags.value.filter((t) => (t as any).blacklisted));

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
  }
  async function removeBlacklist(id: number) {
    await tagsRemoveBlacklist(id);
  }

  function merge(newTags: Tag[]) {
    for (const t of newTags) {
      if (!tags.value.some((x) => x.id === t.id)) {
        tags.value.push(t);
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
    load,
    search,
    setStatus,
    cycle,
    addBlacklist,
    removeBlacklist,
  };
});
