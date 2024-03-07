<template>
  <UForm :schema="formSchema" :state="state" @submit="onSubmit">
    <UFormGroup label="Secret" name="secret" class="mb-3">
      <UTextarea
        v-for="share in state.shares"
        :key="share.id"
        :model-value="share.value"
        @update:model-value="getShareById(share.id).value = $event"
        color="primary"
        variant="outline"
        autoresize
        :maxrows="5"
      />
    </UFormGroup>

    <div class="flex gap-2">
      <UButton @click="addShares()">
        <UIcon name="i-heroicons-plus-20-solid" />
      </UButton>
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

const formSchema = z.object({
  shares: z.array(
    z.object({
      id: z.string().uuid(),
      value: z.string().nonempty(),
    })
  ),
});
type Form = z.infer<typeof formSchema>;

const state = ref<Form>({
  shares: [
    {
      id: uuidv4(),
      value: "",
    },
  ],
});

console.log(state.value);

const secret = ref<string | null>(null);

function getShareById(id: string) {
  return state.value.shares.find((share) => share.id === id);
}

function addShares() {
  state.value.shares.push({
    id: uuidv4(),
    value: "",
  });
}

async function onSubmit() {
  console.log(state.value);

  const { shares } = state.value;

  const response = await invoke<string>("simple_combine", {
    shares: shares.map((share) => share.value),
  });

  console.log(response);

  secret.value = response;
}
</script>
