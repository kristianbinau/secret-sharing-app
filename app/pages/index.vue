<template>
  <UTabs :items="items" class="w-full mt-2">
    <template #encrypt>
      <UnifiedEncrypt class="mt-2" />
    </template>

    <template #decrypt>
      <UnifiedDecrypt class="mt-2" />
    </template>
  </UTabs>
</template>

<script lang="ts" setup>
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
