<template>
  <UForm :schema="formSchema" :state="state" @submit="onSubmit">
    <UFormGroup label="Secret" name="secret" class="mb-3">
      <UTextarea
        v-model="state.secret"
        color="primary"
        variant="outline"
        autoresize
        :maxrows="5"
      />
    </UFormGroup>

    <div>
      <UFormGroup label="Required Shares" name="requiredShares" class="mb-3">
        <UInput v-model="state.requiredShares" type="number" />
      </UFormGroup>

      <UFormGroup label="Generated Shares" name="generatedShares" class="mb-3">
        <UInput v-model="state.generatedShares" type="number" />
      </UFormGroup>
    </div>

    <UButton type="submit">Encrypt</UButton>
  </UForm>

  <!-- Output -->
  <div class="mt-6" v-if="shares !== null">
    <h2 class="mb-2">Shares</h2>

    <div class="flex flex-col gap-3">
      <UTextarea
        v-for="share in shares"
        autoresize
        readonly
        :model-value="share"
      />
    </div>
  </div>
</template>

<script lang="ts" setup>
import { ref } from "vue";
import { z } from "zod";
import { invoke } from "@tauri-apps/api/core";

const formSchema = z
  .object({
    secret: z.string(),
    requiredShares: z.number().int().positive().max(255),
    generatedShares: z.number().int().positive().max(255),
  })
  .refine((schema) => {
    return schema.requiredShares <= schema.generatedShares;
  }, "Generated shares must be greater than or equal to required shares.");
type Form = z.infer<typeof formSchema>;

const state = ref<Form>({ secret: "", requiredShares: 3, generatedShares: 5 });

const shares = ref<string[] | null>(null);

async function onSubmit() {
  const { secret, requiredShares, generatedShares } = state.value;

  const response = await invoke<string[]>("simple_split", {
    secret: secret,
    threshold: requiredShares,
    shares: generatedShares,
  });

  shares.value = response;
}
</script>
