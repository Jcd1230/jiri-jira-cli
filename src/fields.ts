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
  if (field === "key") return issue.key ?? "";
  const val = issue.fields?.[field];
  if (val == null) return "";
  if (typeof val === "string" || typeof val === "number" || typeof val === "boolean")
    return String(val);
  if (val.displayName) return val.displayName;
  if (val.name) return val.name;
  if (val.value) return val.value;
  return JSON.stringify(val);
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
