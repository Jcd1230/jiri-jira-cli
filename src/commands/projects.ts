import { jira } from "../jira";
import { TablePrinter } from "../formatter";
import { CommandNode } from "../types";

export function projects(_: any, printer: TablePrinter): CommandNode {
  return {
    name: "projects",
    description: "List projects visible to the authenticated user.",
    usage: "jiri projects [options]",
    flags: [],
    run: async (_args, opts) => {
      printer.setDefaults(opts);
      const data = await jira.projects();
      const projects = data.values ?? [];
      const rows = [
        ["KEY", "NAME"],
        ...projects.map((p: any) => [p.key as string, p.name as string]),
      ];
      console.log(printer.render(rows));
    },
  };
}
