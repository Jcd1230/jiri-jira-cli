import { jira, FieldLookup } from "../jira";
import { TablePrinter } from "../formatter";
import { suggestFields, formatFieldValues, sortFieldsForDisplay } from "../fields";
import { CommandNode } from "../types";
import { color } from "../colors";
declare const process: any;

export function search(_: any, printer: TablePrinter): CommandNode {
  return {
    name: "search",
    description: "Run a JQL search and list issues.",
    usage: 'jiri search "<JQL>" [options]',
    flags: [
      { flag: "--fields|-f", description: "Comma-separated fields to display (default: key,summary)." },
      { flag: "--get-fields", description: "Show available fields on the first returned issue." },
    ],
    run: async (args, opts) => {
      printer.setDefaults(opts);
      const fields =
        typeof opts.fields === "string"
          ? opts.fields.split(",").map((s: string) => s.trim()).filter(Boolean)
          : ["key", "summary"];
      const jql = args.join(" ");
      if (!jql) {
        console.error(color.red('JQL is required. Example: jiri search "assignee = currentUser()"'));
        process.exit(1);
      }
      const lookup = await jira.fieldLookup();
      const resolved = resolveFields(fields, lookup);
      const data = await jira.search(jql, opts["get-fields"] ? ["*all"] : resolved.queryFields);
      const issues = data.issues ?? [];

      if (opts["get-fields"]) {
        const fieldNames = issues.length ? Object.keys(issues[0].fields ?? {}) : [];
        const rows = sortFieldsForDisplay(fieldNames, lookup).map((f) => {
          const friendly = lookup.idToName[f];
          return friendly ? `"${friendly}" (${f})` : f;
        });
        console.log(printer.render([["FIELD"], ...rows.map((f) => [f])], { header: false }));
        return;
      }

      const headers = resolved.columns.map((c) => c.header.toUpperCase());
      const rows = [headers];
      for (const issue of issues) {
        const row = resolved.columns.map((c) => (c.key ? formatFieldValues(issue, c.key) : ""));
        rows.push(row);
      }
      console.log(printer.render(rows));
    },
  };
}

function resolveFields(
  requested: string[],
  lookup: FieldLookup
): { queryFields: string[]; columns: { header: string; key: string | null }[] } {
  const columns: { header: string; key: string | null }[] = [];
  const queryFields: string[] = [];

  for (const raw of requested) {
    const name = raw.trim();
    const lower = name.toLowerCase();

    if (lookup.idToName[name]) {
      columns.push({ header: lookup.idToName[name] ?? name, key: name });
      queryFields.push(name);
      continue;
    }

    const matchedId = lookup.nameToId[lower];
    if (matchedId) {
      columns.push({ header: name, key: matchedId });
      queryFields.push(matchedId);
      continue;
    }

    suggestFields(name, lookup);
  }

  if (queryFields.length === 0) queryFields.push("key", "summary");

  return { queryFields, columns };
}
