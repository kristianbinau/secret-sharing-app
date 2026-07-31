<template>
  <UForm :state="state" @submit="onSubmit">
    <UFormField label="Shares" name="shares">
      <div v-for="share in state.shares" :key="share.id" class="mb-3">
        <div class="flex items-center gap-2 mb-1">
          <span
            v-if="shareBadges[share.id]"
            class="text-xs px-2 py-0.5 rounded bg-gray-100 dark:bg-gray-800 text-gray-600"
          >
            {{ shareBadges[share.id] }}
          </span>
        </div>
        <UTextarea
          :model-value="share.value"
          @update:model-value="updateShareValueById(share.id, $event)"
          color="primary"
          variant="outline"
          autoresize
          :maxrows="5"
        />
      </div>
    </UFormField>

    <div class="flex gap-2 mb-4">
      <UTooltip text="Remove share field">
        <UButton @click="removeShares()">
          <UIcon name="i-heroicons-minus-20-solid" />
        </UButton>
      </UTooltip>
      <UTooltip text="Add share field">
        <UButton @click="addShares()">
          <UIcon name="i-heroicons-plus-20-solid" />
        </UButton>
      </UTooltip>
      <UButton type="submit" :disabled="!canDecrypt">Decrypt</UButton>
    </div>
  </UForm>

  <div v-if="validShares.length > 0" class="mt-4">
    <h3 class="text-sm font-medium mb-2">Recovery Status</h3>
    <GroupStatus :shares="validShares" />
  </div>

  <div class="mt-6" v-if="secret !== null">
    <h2 class="mb-2">Secret</h2>
    <div class="flex flex-col gap-3">
      <UTextarea autoresize readonly :model-value="secret" />
    </div>
  </div>
</template>

<script lang="ts" setup>
import { ref, computed, reactive } from "vue";
import { v4 as uuidv4 } from "uuid";
import { invoke } from "@tauri-apps/api/core";
import {
  parseShare,
  formatGroupPath,
  canRecover,
  type ParsedShare,
} from "~/utils/nested";

const toast = useToast();

const state = ref({
  shares: [{ id: uuidv4(), value: "" }] as { id: string; value: string }[],
});

const secret = ref<string | null>(null);

const shareBadges = reactive<Record<string, string>>({});

const parsedShares = computed<(ParsedShare | null)[]>(() =>
  state.value.shares.map((s) => parseShare(s.value)),
);

const validShares = computed<ParsedShare[]>(
  () =>
    parsedShares.value.filter(
      (s): s is ParsedShare => s !== null && s.isNested,
    ),
);

const canDecrypt = computed(() => {
  const nonEmpty = state.value.shares.filter((s) => s.value.trim());
  if (nonEmpty.length === 0) return false;
  const nested = validShares.value;
  const hasSimple = nonEmpty.some((s) => {
    const parsed = parseShare(s.value);
    return parsed && !parsed.isNested;
  });
  const hasNested = nested.length > 0;
  if (hasSimple && hasNested) return false;
  if (hasNested) return canRecover(nested);
  return true;
});

watch(
  () => state.value.shares.map((s) => s.value),
  (values) => {
    state.value.shares.forEach((share) => {
      const parsed = parseShare(share.value);
      if (parsed && share.value.trim()) {
        shareBadges[share.id] = formatGroupPath(parsed.path);
      } else if (share.value.trim()) {
        shareBadges[share.id] = "Simple share";
      } else {
        delete shareBadges[share.id];
      }
    });
  },
);

function updateShareValueById(id: string, value: string) {
  const share = state.value.shares.find((s) => s.id === id);
  if (share) {
    share.value = value;
  }
}

function addShares() {
  state.value.shares.push({ id: uuidv4(), value: "" });
}

function removeShares() {
  if (state.value.shares.length > 1) {
    const removed = state.value.shares.pop();
    if (removed) delete shareBadges[removed.id];
  }
}

async function onSubmit() {
  secret.value = null;
  const shares = state.value.shares.map((s) => s.value).filter((s) => s.trim());
  try {
    const response = await invoke<string>("nested_combine", { shares });
    secret.value = response;
  } catch (e: any) {
    toast.add({
      icon: "i-heroicons-exclamation-triangle-20-solid",
      color: "error",
      title: "Decryption failed",
      description: String(e),
      duration: 5000,
    });
  }
}
</script>
