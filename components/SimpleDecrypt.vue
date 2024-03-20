<template>
  <UForm :schema="formSchema" :state="state" @submit="onSubmit">
    <UFormGroup label="Secret" name="secret">
      <UTextarea
        v-for="share in state.shares"
        :key="share.id"
        :model-value="share.value"
        @update:model-value="updateShareValueById(share.id, $event)"
        color="primary"
        variant="outline"
        autoresize
        :maxrows="5"
        class="mb-3"
      />
    </UFormGroup>

    <div class="flex gap-2">
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

      <UButton type="submit">Decrypt</UButton>
    </div>
  </UForm>

  <!-- Output -->
  <div class="mt-6" v-if="secret !== null">
    <h2 class="mb-2">Secret</h2>

    <div class="flex flex-col gap-3">
      <UTextarea autoresize readonly :model-value="secret" />
    </div>
  </div>
</template>

<script lang="ts" setup>
import { ref } from "vue";
import { z } from "zod";
import { v4 as uuidv4 } from "uuid";
import { invoke } from "@tauri-apps/api/core";

/**
 * Schema
 */
const formSchema = z.object({
  shares: z.array(
    z.object({
      id: z.string().uuid(),
      value: z.string().nonempty(),
    })
  ),
});
type Form = z.infer<typeof formSchema>;

/**
 * State
 */
const state = ref<Form>({
  shares: [
    {
      id: uuidv4(),
      value: "",
    },
  ],
});

const secret = ref<string | null>(null);

/**
 * Methods
 */
function updateShareValueById(id: string, value: string) {
  const share = state.value.shares.find((share) => share.id === id);
  if (share) {
    share.value = value;
  }
}

function addShares() {
  state.value.shares.push({
    id: uuidv4(),
    value: "",
  });
}

function removeShares() {
  if (state.value.shares.length > 1) {
    state.value.shares.pop();
  }
}

async function onSubmit() {
  secret.value = null;

  const { shares } = state.value;

  const response = await invoke<string>("simple_combine", {
    shares: shares.map((share) => share.value),
  });

  secret.value = response;
}
</script>
