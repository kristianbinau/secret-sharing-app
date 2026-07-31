<template>
  <UTabs :items="items" class="w-full mt-2">
    <template #encrypt>
      <div class="flex gap-2 mb-4 mt-2">
        <UButton
          size="sm"
          :variant="encryptMode === 'simple' ? 'solid' : 'outline'"
          @click="encryptMode = 'simple'"
        >
          Simple
        </UButton>
        <UButton
          size="sm"
          :variant="encryptMode === 'nested' ? 'solid' : 'outline'"
          @click="encryptMode = 'nested'"
        >
          Nested
        </UButton>
      </div>
      <SimpleEncrypt v-if="encryptMode === 'simple'" />
      <NestedEncrypt v-else />
    </template>

    <template #decrypt>
      <div class="flex gap-2 mb-4 mt-2">
        <UButton
          size="sm"
          :variant="decryptMode === 'simple' ? 'solid' : 'outline'"
          @click="decryptMode = 'simple'"
        >
          Simple
        </UButton>
        <UButton
          size="sm"
          :variant="decryptMode === 'nested' ? 'solid' : 'outline'"
          @click="decryptMode = 'nested'"
        >
          Nested
        </UButton>
      </div>
      <SimpleDecrypt v-if="decryptMode === 'simple'" />
      <NestedDecrypt v-else />
    </template>
  </UTabs>
</template>

<script lang="ts" setup>
import { ref } from "vue";

const encryptMode = ref<"simple" | "nested">("simple");
const decryptMode = ref<"simple" | "nested">("simple");

const items = [
  {
    slot: "encrypt",
    label: "Encrypt",
  },
  {
    slot: "decrypt",
    label: "Decrypt",
  },
];
</script>
