<template>
  <UForm :schema="formSchema" :validate="validate" :state="state" @submit="onSubmit">
    <UFormField label="Secret" name="secret" class="mb-3">
      <UTextarea
        v-model="state.secret"
        color="primary"
        variant="outline"
        autoresize
        :maxrows="5"
      />
    </UFormField>

    <div class="flex gap-3 items-center mb-3">
      <UFormField label="Required Groups" name="threshold">
        <UInput v-model.number="state.threshold" type="number" min="1" max="255" />
      </UFormField>
      <UFormField label="Total Groups" name="groupCount">
        <UInput v-model.number="state.groupCount" type="number" min="1" max="255" />
      </UFormField>
    </div>

    <div class="mb-3">
      <h3 class="text-sm font-medium mb-2">Group Configuration</h3>
      <GroupConfigEditor
        v-model="state.groups"
        :threshold="state.threshold"
        :depth="1"
      />
    </div>

    <div v-if="summary" class="mb-3 text-sm text-gray-500">
      <p>Total shares: {{ totalShares }}</p>
      <p>{{ summary }}</p>
    </div>

    <UButton type="submit">Encrypt</UButton>
  </UForm>

  <div class="mt-6" v-if="shares !== null">
    <h2 class="mb-2">Shares</h2>
    <div v-for="group in groupedShares" :key="group.path" class="mb-4">
      <h3 class="text-sm font-medium mb-2 text-gray-500">{{ group.label }}</h3>
      <div class="flex flex-col gap-3">
        <UTooltip v-for="share in group.shares" text="Click to copy">
          <UTextarea
            autoresize
            readonly
            :model-value="share"
            @click="copyToClipboard(share)"
            class="blur-sm hover:blur-none focus-within:blur-none active:blur-none flex-grow"
          />
        </UTooltip>
      </div>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { ref, computed } from "vue";
import { z } from "zod";
import { invoke } from "@tauri-apps/api/core";
import type { FormError } from "@nuxt/ui";
import {
  type GroupConfig,
  countLeafShares,
  buildAccessStructure,
  parseShare,
  formatGroupPath,
} from "~/utils/nested";

const toast = useToast();

const formSchema = z.object({
  secret: z.string().min(1),
  threshold: z.number().int().positive().max(255),
  groupCount: z.number().int().positive().max(255),
});

type Form = {
  secret: string;
  threshold: number;
  groupCount: number;
  groups: GroupConfig[];
};

const validate = (state: any): FormError[] => {
  const errors: FormError[] = [];
  if (state.groupCount < state.threshold) {
    errors.push({
      name: "groupCount",
      message: "Total groups must be greater or equal to required groups.",
    });
  }
  return errors;
};

function defaultGroups(count: number): GroupConfig[] {
  return Array.from({ length: count }, () => ({
    threshold: 1,
    count: 3,
    groups: [],
  }));
}

const state = ref<Form>({
  secret: "",
  threshold: 1,
  groupCount: 2,
  groups: defaultGroups(2),
});

const shares = ref<string[] | null>(null);

watch(
  () => state.value.groupCount,
  (newCount, oldCount) => {
    if (newCount !== oldCount) {
      const groups = state.value.groups;
      if (newCount > groups.length) {
        while (groups.length < newCount) {
          groups.push({ threshold: 1, count: 3, groups: [] });
        }
      } else if (newCount < groups.length) {
        groups.splice(newCount);
      }
    }
  },
);

watch(
  () => state.value.groups.length,
  (newLen) => {
    state.value.groupCount = newLen;
  },
);

const totalShares = computed(() => countLeafShares(state.value.groups));

const summary = computed(() => {
  if (state.value.groups.length === 0) return "";
  return buildAccessStructure(state.value.threshold, state.value.groups);
});

const groupedShares = computed(() => {
  if (!shares.value) return [];
  const map = new Map<string, { path: string; label: string; shares: string[] }>();
  for (const share of shares.value) {
    const parsed = parseShare(share);
    const pathKey = parsed ? parsed.path.join(".") : "simple";
    const label = parsed ? formatGroupPath(parsed.path) : "Shares";
    if (!map.has(pathKey)) {
      map.set(pathKey, { path: pathKey, label, shares: [] });
    }
    map.get(pathKey)!.shares.push(share);
  }
  return Array.from(map.values());
});

async function onSubmit() {
  const { secret, threshold, groups } = state.value;
  try {
    shares.value = null;
    const response = await invoke<string[]>("nested_split", {
      secret: secret,
      threshold: threshold,
      groups: groups,
    });
    shares.value = response;
  } catch (e: any) {
    toast.add({
      icon: "i-heroicons-exclamation-triangle-20-solid",
      color: "error",
      title: "Encryption failed",
      description: String(e),
      duration: 5000,
    });
  }
}

function copyToClipboard(text: string) {
  navigator.clipboard.writeText(text);
  toast.add({
    icon: "i-heroicons-clipboard-document-check-20-solid",
    title: "Copied",
    description: "The share has been copied to the clipboard.",
    duration: 2000,
  });
}
</script>
