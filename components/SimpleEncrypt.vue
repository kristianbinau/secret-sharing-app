<template>
  <UForm :schema="formSchema" :validate="validate" :state="state" @submit="onSubmit">
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
      <UTooltip v-for="share in shares" text="Click to copy">
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
</template>

<script lang="ts" setup>
import { ref } from "vue";
import { z } from "zod";
import { invoke } from "@tauri-apps/api/core";
import type { FormError } from "#ui/types";

const toast = useToast();

/**
 * Schema
 */
const formSchema = z.object({
  secret: z.string(),
  requiredShares: z.number().int().positive().max(255),
  generatedShares: z.number().int().positive().max(255),
});
type Form = z.infer<typeof formSchema>;

const validate = (state: any): FormError[] => {
  const errors = [];
  if (state.generatedShares < state.requiredShares) {
    errors.push({
      path: "generatedShares",
      message: "Generated shares must be greater or equal to required shares.",
    });
  }
  return errors;
};

/**
 * State
 */
const state = ref<Form>({ secret: "", requiredShares: 3, generatedShares: 5 });
const shares = ref<string[] | null>(null);

/**
 * Methods
 */
async function onSubmit() {
  const { secret, requiredShares, generatedShares } = state.value;

  const response = await invoke<string[]>("simple_split", {
    secret: secret,
    threshold: requiredShares,
    shares: generatedShares,
  });

  shares.value = response;
}

function copyToClipboard(text: string) {
  navigator.clipboard.writeText(text);

  // Show a toast
  toast.add({
    icon: "i-heroicons-clipboard-document-check-20-solid",
    title: "Copied",
    description: "The share has been copied to the clipboard.",
    timeout: 2000,
  });
}
</script>
