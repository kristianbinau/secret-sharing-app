<template>
  <div class="flex flex-col gap-3">
    <div
      v-for="(group, index) in groups"
      :key="index"
      class="border-l-2 border-gray-300 dark:border-gray-700 pl-4"
    >
      <div class="flex flex-wrap gap-2 items-center mb-2">
        <span class="text-sm font-medium text-gray-500">
          {{ path ? `${path}.${index + 1}` : `Group ${index + 1}` }}
        </span>
        <span class="text-sm text-gray-400">Require</span>
        <UInput
          v-model.number="group.threshold"
          type="number"
          min="1"
          max="255"
          class="w-20"
          size="sm"
        />
        <span class="text-sm text-gray-400">of</span>
        <UInput
          v-model.number="group.count"
          type="number"
          min="1"
          max="255"
          class="w-20"
          size="sm"
          :disabled="group.groups.length > 0"
        />
        <UButton
          size="xs"
          :variant="group.groups.length === 0 ? 'solid' : 'outline'"
          @click="switchMode(index, 'shares')"
        >
          Shares
        </UButton>
        <UButton
          size="xs"
          :variant="group.groups.length > 0 ? 'solid' : 'outline'"
          @click="switchMode(index, 'subgroups')"
        >
          Sub-groups
        </UButton>
        <UButton
          v-if="groups.length > 1"
          size="xs"
          color="error"
          variant="ghost"
          @click="removeGroup(index)"
        >
          <UIcon name="i-heroicons-trash-20-solid" />
        </UButton>
      </div>

      <GroupConfigEditor
        v-if="group.groups.length > 0"
        v-model="group.groups"
        :threshold="group.threshold"
        :depth="depth + 1"
        :path="path ? `${path}.${index + 1}` : `${index + 1}`"
      />
    </div>

    <UButton
      size="sm"
      variant="outline"
      @click="addGroup"
    >
      <UIcon name="i-heroicons-plus-20-solid" />
      <span>Add Group</span>
    </UButton>
  </div>
</template>

<script lang="ts" setup>
import type { GroupConfig } from "~/utils/nested";

const groups = defineModel<GroupConfig[]>({ required: true });

const props = withDefaults(
  defineProps<{
    threshold: number;
    depth?: number;
    path?: string;
  }>(),
  {
    depth: 1,
    path: "",
  },
);

function addGroup() {
  groups.value.push({
    threshold: 1,
    count: 3,
    groups: [],
  });
}

function removeGroup(index: number) {
  if (groups.value.length > 1) {
    groups.value.splice(index, 1);
  }
}

function switchMode(index: number, mode: "shares" | "subgroups") {
  const group = groups.value[index];
  if (!group) return;
  if (mode === "shares") {
    group.groups = [];
  } else {
    if (group.groups.length === 0) {
      group.groups = [
        { threshold: 1, count: 2, groups: [] },
        { threshold: 1, count: 2, groups: [] },
      ];
      group.count = group.groups.length;
    }
  }
}

watch(
  () => groups.value.map((g) => g.groups.length),
  () => {
    groups.value.forEach((group) => {
      if (group.groups.length > 0) {
        group.count = group.groups.length;
      }
    });
  },
);
</script>
