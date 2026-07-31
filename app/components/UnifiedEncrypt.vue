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

    <template v-if="isSimple">
      <UFormField label="Required Shares" name="requiredShares" class="mb-3">
        <UInput v-model.number="state.groups[0]!.threshold" type="number" min="1" max="255" />
      </UFormField>

      <UFormField label="Generated Shares" name="generatedShares" class="mb-3">
        <UInput v-model.number="state.groups[0]!.count" type="number" min="1" max="255" />
      </UFormField>

      <UButton variant="outline" @click="addGroup">
        <UIcon name="i-heroicons-plus-20-solid" />
        <span>Add Group</span>
      </UButton>
    </template>

    <template v-else>
      <div class="flex gap-3 items-center mb-3">
        <UFormField label="Required Groups" name="topThreshold">
          <UInput v-model.number="state.topThreshold" type="number" min="1" max="255" />
        </UFormField>
      </div>

      <div class="mb-3">
        <h3 class="text-sm font-medium mb-2">Group Configuration</h3>
        <GroupConfigEditor
          v-model="state.groups"
          :threshold="state.topThreshold"
          :depth="1"
        />
      </div>
    </template>

    <div v-if="summary" class="mb-3 text-sm text-gray-500 mt-3">
      <p>Total shares: {{ totalShares }}</p>
      <p>{{ summary }}</p>
    </div>

    <UButton type="submit" class="mt-3">Encrypt</UButton>
  </UForm>

  <div class="mt-6" v-if="shares !== null">
    <h2 class="mb-2">Shares</h2>
    <div v-for="group in groupedShares" :key="group.path" class="mb-4">
      <h3 v-if="groupedShares.length > 1" class="text-sm font-medium mb-2 text-gray-500">{{ group.label }}</h3>
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
import { ref, computed, watch } from "vue";
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
});

type Form = {
  secret: string;
  topThreshold: number;
  groups: GroupConfig[];
};

const state = ref<Form>({
  secret: "",
  topThreshold: 1,
  groups: [{ threshold: 2, count: 5, groups: [] }],
});

const shares = ref<string[] | null>(null);

const isSimple = computed(
  () =>
    state.value.groups.length === 1 &&
    state.value.groups[0]!.groups.length === 0,
);

watch(isSimple, (simple) => {
  if (simple) {
    state.value.topThreshold = 1;
  }
});

const totalShares = computed(() => countLeafShares(state.value.groups));

const summary = computed(() => {
  if (isSimple.value) {
    const g = state.value.groups[0]!;
    return `${g.threshold} of ${g.count} shares required`;
  }
  return buildAccessStructure(state.value.topThreshold, state.value.groups);
});

const validate = (state: any): FormError[] => {
  const errors: FormError[] = [];
  const simple =
    state.groups.length === 1 && state.groups[0].groups.length === 0;
  if (simple) {
    if (state.groups[0].count < state.groups[0].threshold) {
      errors.push({
        name: "generatedShares",
        message: "Generated shares must be greater or equal to required shares.",
      });
    }
  } else {
    if (state.groups.length < state.topThreshold) {
      errors.push({
        name: "topThreshold",
        message: "Required groups must be less than or equal to total groups.",
      });
    }
  }
  return errors;
};

function addGroup() {
  state.value.groups.push({ threshold: 1, count: 3, groups: [] });
}

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
  const { secret, topThreshold, groups } = state.value;
  try {
    shares.value = null;
    if (isSimple.value) {
      const response = await invoke<string[]>("simple_split", {
        secret: secret,
        threshold: groups[0]!.threshold,
        shares: groups[0]!.count,
      });
      shares.value = response;
    } else {
      const response = await invoke<string[]>("nested_split", {
        secret: secret,
        threshold: topThreshold,
        groups: groups,
      });
      shares.value = response;
    }
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
