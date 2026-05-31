use comfy_table::Table;
use serde_json::Value;

/// Supported output formats for the CLI.
pub enum OutputFormat {
    /// Pretty-printed table with borders.
    Table,
    /// Comma-separated values.
    CSV,
    /// Pretty-printed JSON (array of objects with header keys).
    Json,
    /// One JSON object per line (newline-delimited JSON).
    Jsonl,
    /// GitHub-flavored Markdown table.
    Markdown,
    /// Space-padded columns without borders.
    Plain,
}

/// Helper for rendering data in various formats.
pub struct Formatter {
    pub format: OutputFormat,
    pub no_header: bool,
}

impl Formatter {
    /// Create a new Formatter with the specified format and header preference.
    pub fn new(format: OutputFormat, no_header: bool) -> Self {
        Self { format, no_header }
    }

    /// Render a grid of rows into a formatted string.
    pub fn render(&self, rows: Vec<Vec<String>>) -> String {
        match self.format {
            OutputFormat::Table => self.render_table(rows),
            OutputFormat::CSV => self.render_csv(rows),
            OutputFormat::Json => self.render_json(rows),
            OutputFormat::Jsonl => self.render_jsonl(rows),
            OutputFormat::Markdown => self.render_markdown(rows),
            OutputFormat::Plain => self.render_plain(rows),
        }
    }

    fn render_table(&self, rows: Vec<Vec<String>>) -> String {
        let mut table = Table::new();
        table.load_preset(comfy_table::presets::UTF8_FULL);

        if rows.is_empty() {
            return String::new();
        }

        let mut iter = rows.into_iter();
        if self.no_header {
            iter.next(); // Discard header
        } else if let Some(header) = iter.next() {
            table.set_header(header);
        }

        for row in iter {
            table.add_row(row);
        }

        table.to_string()
    }

    fn render_csv(&self, rows: Vec<Vec<String>>) -> String {
        let mut iter = rows.into_iter();
        let first_row = iter.next();
        let rows_to_render = if self.no_header {
            iter.collect::<Vec<_>>()
        } else {
            first_row.into_iter().chain(iter).collect::<Vec<_>>()
        };

        rows_to_render
            .into_iter()
            .map(|row| {
                row.iter()
                    .map(|cell| {
                        // Simple CSV quoting logic
                        if cell.contains(',') || cell.contains('"') || cell.contains('\n') {
                            format!("\"{}\"", cell.replace('"', "\"\""))
                        } else {
                            cell.clone()
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(",")
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn render_json(&self, rows: Vec<Vec<String>>) -> String {
        if rows.is_empty() {
            return "[]".to_string();
        }

        let mut iter = rows.into_iter();
        let headers = match iter.next() {
            Some(h) => h,
            None => return "[]".to_string(),
        };

        let objects: Vec<Value> = iter
            .map(|row| {
                let obj: serde_json::Map<String, Value> = headers
                    .iter()
                    .zip(row.iter())
                    .map(|(k, v)| (k.clone(), Value::String(v.clone())))
                    .collect();
                Value::Object(obj)
            })
            .collect();

        serde_json::to_string_pretty(&objects).unwrap_or_default()
    }

    fn render_jsonl(&self, rows: Vec<Vec<String>>) -> String {
        if rows.is_empty() {
            return String::new();
        }

        let mut iter = rows.into_iter();
        let headers = match iter.next() {
            Some(h) => h,
            None => return String::new(),
        };

        iter.map(|row| {
            let obj: serde_json::Map<String, Value> = headers
                .iter()
                .zip(row.iter())
                .map(|(k, v)| (k.clone(), Value::String(v.clone())))
                .collect();
            serde_json::to_string(&Value::Object(obj)).unwrap_or_default()
        })
        .collect::<Vec<_>>()
        .join("\n")
    }

    fn render_markdown(&self, rows: Vec<Vec<String>>) -> String {
        if rows.is_empty() {
            return String::new();
        }

        let mut iter = rows.into_iter();
        let mut lines = Vec::new();

        if self.no_header {
            iter.next(); // Discard header but still build separator
        }

        let header = match iter.next() {
            Some(h) => h,
            None => return String::new(),
        };

        // Header row
        lines.push(format!("| {} |", header.join(" | ")));
        // Separator row
        let sep: Vec<String> = header.iter().map(|_| "---".to_string()).collect();
        lines.push(format!("| {} |", sep.join(" | ")));

        // Data rows
        for row in iter {
            // Escape pipe characters in cell values
            let escaped: Vec<String> = row.iter().map(|c| c.replace('|', "\\|")).collect();
            lines.push(format!("| {} |", escaped.join(" | ")));
        }

        lines.join("\n")
    }

    fn render_plain(&self, rows: Vec<Vec<String>>) -> String {
        // Simple space-padded output
        if rows.is_empty() {
            return String::new();
        }

        let mut iter = rows.into_iter();
        let rows_to_render = if self.no_header {
            iter.next();
            iter.collect::<Vec<_>>()
        } else {
            iter.collect::<Vec<_>>()
        };

        if rows_to_render.is_empty() {
            return String::new();
        }

        let num_cols = rows_to_render[0].len();
        let mut col_widths = vec![0; num_cols];

        for row in &rows_to_render {
            for (i, cell) in row.iter().enumerate() {
                col_widths[i] = col_widths[i].max(cell.len());
            }
        }

        rows_to_render
            .iter()
            .map(|row| {
                row.iter()
                    .enumerate()
                    .map(|(i, cell)| format!("{:width$}", cell, width = col_widths[i]))
                    .collect::<Vec<_>>()
                    .join("  ")
                    .trim_end()
                    .to_string()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}
