export type TableOptions = { csv?: boolean; plain?: boolean; header?: boolean };

export class TablePrinter {
  defaults: TableOptions = { csv: false, plain: false, header: true };

  setDefaults(opts: Record<string, any>) {
    if ("csv" in opts) this.defaults.csv = opts.csv;
    if ("plain" in opts) this.defaults.plain = opts.plain;
    if ("header" in opts) this.defaults.header = opts.header;
  }

  render(rows: string[][], overrides: TableOptions = {}): string {
    return formatTable(rows, { ...this.defaults, ...overrides });
  }
}

export function formatTable(rows: string[][], opts: TableOptions = { csv: false, plain: false, header: true }): string {
  if (rows.length === 0) return "";

  const bodyRows = opts.header ? rows : rows.slice(1);
  if (bodyRows.length === 0) return "";

  if (opts.csv) return toCSV(opts.header ? rows : bodyRows);

  const colCount = Math.max(...rows.map((r) => r.length));
  const widths = Array(colCount).fill(0);

  for (const row of bodyRows) {
    for (let i = 0; i < colCount; i++) widths[i] = Math.max(widths[i], (row[i] ?? "").length);
  }

  if (opts.plain) {
    const renderPlain = (row: string[], isHeader = false) =>
      row
        .map((cell, i) => {
          const text = (cell ?? "").padEnd(widths[i], " ");
          const content = ` ${text} `;
          return isHeader && opts.header ? bold(dim(content)) : content;
        })
        .join(" ");
    const [headerRow, ...body] = opts.header ? rows : bodyRows;
    return [
      opts.header ? renderPlain(headerRow, true) : null,
      ...body.map((r) => renderPlain(r)),
    ]
      .filter(Boolean)
      .join("\n");
  }

  const top = "┌" + widths.map((w) => "─".repeat(w + 2)).join("┬") + "┐";
  const mid = "├" + widths.map((w) => "─".repeat(w + 2)).join("┼") + "┤";
  const bottom = "└" + widths.map((w) => "─".repeat(w + 2)).join("┴") + "┘";

  const renderRow = (row: string[], isHeader = false) =>
    "│" +
    row
      .map((cell, i) => {
        const text = (cell ?? "").padEnd(widths[i], " ");
        const content = ` ${text} `;
        return isHeader && opts.header ? bold(dim(content)) : content;
      })
      .join("│") +
    "│";

  const [headerRow, ...body] = opts.header ? rows : bodyRows;
  return [
    top,
    opts.header ? renderRow(headerRow, true) : null,
    opts.header ? mid : null,
    ...body.map((r) => renderRow(r)),
    bottom,
  ]
    .filter(Boolean)
    .join("\n");
}

function toCSV(rows: string[][]): string {
  const esc = (v: string) => {
    const needsQuote = /[",\n]/.test(v);
    const val = v.replace(/"/g, '""');
    return needsQuote ? `"${val}"` : val;
  };
  return rows.map((r) => r.map((c) => esc(String(c ?? ""))).join(",")).join("\n");
}

// minimal color helpers to avoid circular dep
const bold = (s: string) => `\u001b[1m${s}\u001b[0m`;
const dim = (s: string) => `\u001b[2m${s}\u001b[0m`;
