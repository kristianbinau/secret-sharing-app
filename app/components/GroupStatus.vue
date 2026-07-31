<template>
  <div class="text-sm">
    <div v-if="level === 0 && topSummary" class="mb-3 p-2 rounded bg-gray-50 dark:bg-gray-800">
      <span class="font-medium">{{ topSummary }}</span>
    </div>
    <div v-for="group in groups" :key="group.index" class="ml-4 mb-1">
      <div class="flex items-center gap-2">
        <UIcon
          v-if="group.ready"
          name="i-heroicons-check-circle-20-solid"
          class="text-green-500"
        />
        <UIcon v-else name="i-heroicons-x-circle-20-solid" class="text-red-400" />
        <span>
          Group {{ group.index }}: {{ group.current }}/{{ group.threshold }}
          <span v-if="group.isLeaf"> shares</span>
          <span v-else> sub-groups ready</span>
        </span>
      </div>
      <GroupStatus
        v-if="!group.isLeaf && group.children.length > 0"
        :shares="group.shares"
        :level="level + 1"
      />
    </div>
  </div>
</template>

<script lang="ts" setup>
import { buildStatusTree, type ParsedShare, type StatusNode } from "~/utils/nested";

const props = withDefaults(
  defineProps<{
    shares: ParsedShare[];
    level?: number;
  }>(),
  {
    level: 0,
  },
);

const topThreshold = computed(() => props.shares[0]?.thresholds[0] ?? 0);

const topSummary = computed(() => {
  if (props.shares.length === 0) return null;
  const readyCount = groups.value.filter((g) => g.ready).length;
  return `Groups ready: ${readyCount} of ${topThreshold.value} required (${groups.value.length} total)`;
});

const groups = computed<StatusNode[]>(() =>
  buildStatusTree(props.shares, props.level),
);
</script>
