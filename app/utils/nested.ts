export interface GroupConfig {
  threshold: number;
  count: number;
  groups: GroupConfig[];
}

export interface ParsedShare {
  isNested: boolean;
  path: number[];
  thresholds: number[];
  leafThreshold: number;
}

export function base64UrlDecode(str: string): Uint8Array | null {
  try {
    let base64 = str.replace(/-/g, "+").replace(/_/g, "/");
    while (base64.length % 4) base64 += "=";
    const binary = atob(base64);
    const bytes = new Uint8Array(binary.length);
    for (let i = 0; i < binary.length; i++) {
      bytes[i] = binary.charCodeAt(i);
    }
    return bytes;
  } catch {
    return null;
  }
}

export function parseShare(str: string): ParsedShare | null {
  if (!str) return null;
  const bytes = base64UrlDecode(str);
  if (!bytes || bytes.length < 2) return null;
  if (bytes[0] !== 0x00) {
    return { isNested: false, path: [], thresholds: [], leafThreshold: 0 };
  }
  const depth = bytes[1]!;
  if (depth < 1) return null;
  const thresholds: number[] = [];
  const path: number[] = [];
  let offset = 2;
  for (let i = 0; i < depth; i++) {
    if (offset + 1 >= bytes.length) return null;
    thresholds.push(bytes[offset]!);
    path.push(bytes[offset + 1]!);
    offset += 2;
  }
  if (offset >= bytes.length) return null;
  const leafThreshold = bytes[offset]!;
  return { isNested: true, path, thresholds, leafThreshold };
}

export function formatGroupPath(path: number[]): string {
  if (path.length === 0) return "Simple";
  return "Group " + path.join(".");
}

export function countLeafShares(groups: GroupConfig[]): number {
  return groups.reduce((total, group) => {
    if (group.groups.length === 0) {
      return total + group.count;
    }
    return total + countLeafShares(group.groups);
  }, 0);
}

export function buildAccessStructure(
  threshold: number,
  groups: GroupConfig[],
): string {
  if (groups.length === 0) return "";
  const parts = groups.map((group) => {
    if (group.groups.length === 0) {
      return `${group.threshold} of ${group.count} shares`;
    }
    return `${group.threshold} of ${group.count} sub-groups (${buildAccessStructure(
      group.threshold,
      group.groups,
    )})`;
  });
  return `${threshold} of ${groups.length} groups: ${parts.join(", ")}`;
}

export interface StatusNode {
  index: number;
  current: number;
  threshold: number;
  ready: boolean;
  isLeaf: boolean;
  shares: ParsedShare[];
  children: StatusNode[];
}

export function buildStatusTree(
  shares: ParsedShare[],
  level: number,
): StatusNode[] {
  const depth = shares[0]?.thresholds.length ?? 0;
  if (depth === 0) return [];
  const leaf = level === depth - 1;

  const map = new Map<number, ParsedShare[]>();
  for (const share of shares) {
    const idx = share.path[level]!;
    if (!map.has(idx)) map.set(idx, []);
    map.get(idx)!.push(share);
  }

  return Array.from(map.entries())
    .sort((a, b) => a[0] - b[0])
    .map(([_index, groupShares], seqIndex) => {
      const threshold = leaf
        ? groupShares[0]!.leafThreshold
        : groupShares[0]!.thresholds[level + 1]!;

      if (leaf) {
        return {
          index: seqIndex + 1,
          current: groupShares.length,
          threshold,
          ready: groupShares.length >= threshold,
          isLeaf: true,
          shares: groupShares,
          children: [],
        } satisfies StatusNode;
      }

      const children = buildStatusTree(groupShares, level + 1);
      const readyCount = children.filter((c) => c.ready).length;
      return {
        index: seqIndex + 1,
        current: readyCount,
        threshold,
        ready: readyCount >= threshold,
        isLeaf: false,
        shares: groupShares,
        children,
      } satisfies StatusNode;
    });
}

export function canRecover(shares: ParsedShare[]): boolean {
  if (shares.length === 0) return false;
  const depth = shares[0]!.thresholds.length;
  if (depth === 0) return false;
  if (!shares.every((s) => s.thresholds.length === depth)) return false;
  const tree = buildStatusTree(shares, 0);
  const topThreshold = shares[0]!.thresholds[0]!;
  const readyCount = tree.filter((g) => g.ready).length;
  return readyCount >= topThreshold;
}
