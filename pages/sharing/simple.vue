<template>
  <UTabs :items="items" class="w-full">
    <template #encrypt="{ item }">
      <SimpleEncrypt />
    </template>

    <template #decrypt="{ item }">
      <SimpleDecrypt />
    </template>
  </UTabs>
</template>

<script lang="ts" setup>
import { ref } from "vue";
import { z } from "zod";
import { invoke } from "@tauri-apps/api/core";

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
