import { FieldLookup } from "./jira";
import { color } from "./colors";

export function suggestFields(input: string, lookup: FieldLookup, limit = 3, maxDistance = 3) {
  const picks = Object.values(lookup.idToName)
    .map((name) => ({ name, score: levenshtein(input.toLowerCase(), name.toLowerCase()) }))
    .filter((s) => s.score <= maxDistance)
    .sort((a, b) => a.score - b.score)
    .slice(0, limit)
    .map((s) => s.name);

  if (!picks.length) {
    console.error(color.yellow(`Field '${input}' not found.`));
    return;
  }
  console.error(color.yellow(`Field '${input}' not found. Did you mean: ${picks.join(", ")}?`));
}

export function formatFieldValues(issue: any, field: string): string {
  if (field === "key" || field === "issuekey") return issue.key ?? "";
  const val = issue.fields?.[field];
  return normalizeValue(val);
}

function normalizeValue(val: any): string {
  if (val == null) return "";
  if (typeof val === "string" || typeof val === "number" || typeof val === "boolean") return String(val);

  if (Array.isArray(val)) {
    const parts = val.map((v) => normalizeValue(v)).filter((v) => v !== "");
    return parts.join(", ");
  }

  // Common Jira entity shapes
  if (val.displayName) return val.displayName;
  if (val.name) return val.name;
  if (val.value) return val.value;
  if (val.title) return val.title;
  if (val.label) return val.label;
  if (val.key) return val.key;

  // Nested option (e.g., {child, parent})
  if (val.child) return normalizeValue(val.child);
  if (val.parent) return normalizeValue(val.parent);

  return JSON.stringify(val);
}

export function sortFieldsForDisplay(fieldIds: string[], lookup: FieldLookup): string[] {
  const friendly = (id: string) => lookup.idToName[id] ?? id;
  const isCustom = (id: string) => id.startsWith("customfield_");
  return [...fieldIds].sort((a, b) => {
    const customDiff = Number(isCustom(a)) - Number(isCustom(b));
    if (customDiff !== 0) return customDiff; // system first, then custom
    return friendly(a).localeCompare(friendly(b), undefined, { sensitivity: "base" });
  });
}

function levenshtein(a: string, b: string): number {
  const dp = Array.from({ length: a.length + 1 }, () => Array(b.length + 1).fill(0));
  for (let i = 0; i <= a.length; i++) dp[i][0] = i;
  for (let j = 0; j <= b.length; j++) dp[0][j] = j;
  for (let i = 1; i <= a.length; i++) {
    for (let j = 1; j <= b.length; j++) {
      const cost = a[i - 1] === b[j - 1] ? 0 : 1;
      dp[i][j] = Math.min(
        dp[i - 1][j] + 1,
        dp[i][j - 1] + 1,
        dp[i - 1][j - 1] + cost
      );
    }
  }
  return dp[a.length][b.length];
}
