declare const process: any;

export type ArgvSplit = {
  command: string | undefined;
  flags: string[];
  positionals: string[];
};

function splitArgs(argv: string[]): ArgvSplit {
  const idx = argv.findIndex((a) => !a.startsWith("-"));
  if (idx === -1) return { command: undefined, flags: argv, positionals: [] };
  return {
    command: argv[idx],
    flags: argv.slice(0, idx),
    positionals: argv.slice(idx + 1),
  };
}

export const argvSplit = splitArgs(process.argv.slice(2));
