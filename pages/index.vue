<template>
  <UTabs :items="items" class="w-full mt-2">
    <template #encrypt="{ item }">
      <SimpleEncrypt />
    </template>

    <template #decrypt="{ item }">
      <SimpleDecrypt />
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
