export type OptionDef = {
  flag: string;
  aliases?: string[];
  description: string;
  value?: any;
};

export type CommandNode = {
  name: string | null;
  description: string;
  usage: string;
  flags: OptionDef[];
  flagHeader?: string;
  run?: (args: string[], options: Record<string, any>) => Promise<void>;
  subcommands?: CommandNode[];
};
