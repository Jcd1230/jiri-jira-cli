#!/usr/bin/env bun
/// <reference lib="es2021" />
/// <reference lib="dom" />
/// <reference lib="dom.iterable" />

import { TablePrinter } from "./src/formatter";
import { color } from "./src/colors";
import { projects } from "./src/commands/projects";
import { search } from "./src/commands/search";
import { CommandNode, OptionDef } from "./src/types";
import { env } from "./src/env";
import { argvSplit } from "./src/argv";

const printer = new TablePrinter();

// --- CLI definition ---
const outputOptionDefs: OptionDef[] = [
  { flag: "--csv", description: "Output comma-separated values (no borders)." },
  { flag: "--plain", description: "No borders, padded columns." },
  { flag: "--no-header", description: "Omit header row.", value: false },
  { flag: "--help|-h", description: "Show help." },
];

const cli: CommandNode = {
  name: null,
  description: "Minimal Jira CLI",
  usage: "jiri <command> [options]",
  flagHeader: "Output options (table/CSV)",
  flags: [...outputOptionDefs],
  subcommands: [projects(env, printer), search(env, printer)],
};

// --- main ---
const parsedArgs = parseCli(argvSplit);

async function main() {
  if (parsedArgs.help) {
    printHelp(parsedArgs.commandName ?? undefined);
    return;
  }
  if (parsedArgs.error) {
    console.error(color.red(parsedArgs.error));
    printHelp(parsedArgs.commandName ?? undefined);
    process.exit(1);
  }
  if (!parsedArgs.run) {
    usage();
    process.exit(1);
  }
  printer.setDefaults(parsedArgs.opts);
  await parsedArgs.run(parsedArgs.args, parsedArgs.opts);
}

main().catch((err) => {
  console.error(err instanceof Error ? err.message : err);
  process.exit(1);
});

function parseFlags(
  args: string[],
  defs: OptionDef[],
  initial: Record<string, any> = {}
): { options: Record<string, any>; rest: string[] } {
  const opts: Record<string, any> = { ...initial };
  const rest: string[] = [];

  const expandedDefs = defs.map((d) => {
    if (d.aliases && d.aliases.length) return d;
    const parts = d.flag.split("|");
    return { ...d, flag: parts[0], aliases: parts.slice(1) };
  });

  for (let i = 0; i < args.length; i++) {
    const arg = args[i];
    if (!arg.startsWith("-")) {
      rest.push(arg);
      continue;
    }

    const [flagOnly, inlineVal] = arg.split("=", 2);
    const def = expandedDefs.find((d) => d.flag === flagOnly || d.aliases?.includes(flagOnly));
    if (!def) {
      rest.push(arg);
      continue;
    }

    const field = flagOnly.replace(/^-+/, "");
    let value: any = def.value !== undefined ? def.value : true;

    if (inlineVal !== undefined) value = inlineVal;
    else if (args[i + 1] && !args[i + 1].startsWith("-")) {
      value = args[++i];
    }

    opts[field] = value;
  }

  return { options: opts, rest };
}

function parseCli(split: { command: string | undefined; flags: string[]; positionals: string[] }) {
  const { command: commandName, flags, positionals } = split;

  const wantsRootHelp = (!commandName && (flags.includes("--help") || flags.includes("-h"))) ||
    commandName === "--help" ||
    commandName === "-h";
  if (wantsRootHelp) return { help: true, run: null, commandName: null, opts: {}, args: [], error: null };

  if (!commandName) return { help: false, run: null, commandName, opts: {}, args: [], error: "Unknown command." };

  const cmd = cli.subcommands?.find((c) => c.name === commandName);
  if (!cmd || !cmd.run) return { help: false, run: null, commandName, opts: {}, args: [], error: `Unknown command '${commandName}'.` };

  const combinedDefs = [...cli.flags, ...cmd.flags];
  const { options: opts, rest } = parseFlags([...flags, ...positionals], combinedDefs, {});
  const unknownFlag = rest.find((a) => a.startsWith("-"));
  const help = !!opts.help;
  return { help, run: cmd.run, commandName, opts, args: rest.filter((a) => !a.startsWith("-")), error: unknownFlag ? `Unknown option '${unknownFlag}'.` : null };
}

function usage() {
  printHelp();
}

function printHelp(command?: string) {
  const node = command ? cli.subcommands?.find((c) => c.name === command) : cli;
  if (!node) return;

  const title = color.bold(color.cyan(`jiri${node.name ? " " + node.name : " - minimal Jira CLI"}`));
  const usageLine = node.usage || "jiri <command> [options]";

  const sections: string[] = [];
  if (node.flags?.length) {
    sections.push(renderOptionList(node.flags, node.flagHeader || `${capitalize(node.name || "Global")} Flags`));
  }
  if (node !== cli && cli.flags.length) {
    sections.push(renderOptionList(cli.flags, cli.flagHeader || "Global Flags"));
  }

  const commandsBlock =
    node === cli && node.subcommands?.length
      ? `${color.bold("Commands")}:\n${node.subcommands
          .map((def) => `  ${color.green((def.name ?? "").padEnd(10, " "))} ${def.description}`)
          .join("\n")}\n\n`
      : "";

  const descriptionBlock = node.description ? `${color.bold("Description")}:\n  ${node.description}\n\n` : "";

  console.log(
    `${title}\n\n${color.bold("Usage")}:\n  ${usageLine}\n\n${commandsBlock}${descriptionBlock}${sections
      .filter(Boolean)
      .join("\n\n")}\n`
  );
}

function renderOptionList(defs: OptionDef[], header?: string): string {
  const rows = defs.map((d) => {
    const flags = [d.flag, ...(d.aliases ?? [])].join(", ");
    return `  ${color.yellow(flags.padEnd(14, " "))} ${d.description}`;
  });
  if (rows.length === 0) return "";
  const title = header ? `${color.bold(header)}:\n` : "";
  return `${title}${rows.join("\n")}`;
}

function capitalize(s: string) {
  return s.charAt(0).toUpperCase() + s.slice(1);
}
declare const process: any;
