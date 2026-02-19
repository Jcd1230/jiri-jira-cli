use comfy_table::Table;

pub enum OutputFormat {
    Table,
    CSV,
    Plain,
}

pub struct Formatter {
    pub format: OutputFormat,
    pub no_header: bool,
}

impl Formatter {
    pub fn new(format: OutputFormat, no_header: bool) -> Self {
        Self { format, no_header }
    }

    pub fn render(&self, rows: Vec<Vec<String>>) -> String {
        match self.format {
            OutputFormat::Table => self.render_table(rows),
            OutputFormat::CSV => self.render_csv(rows),
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
        if !self.no_header {
            if let Some(header) = iter.next() {
                table.set_header(header);
            }
        }

        for row in iter {
            table.add_row(row);
        }

        table.to_string()
    }

    fn render_csv(&self, rows: Vec<Vec<String>>) -> String {
        rows.iter()
            .map(|row| {
                row.iter()
                    .map(|cell| {
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

    fn render_plain(&self, rows: Vec<Vec<String>>) -> String {
        // Simple space-padded output
        if rows.is_empty() {
            return String::new();
        }

        let num_cols = rows[0].len();
        let mut col_widths = vec![0; num_cols];

        for row in &rows {
            for (i, cell) in row.iter().enumerate() {
                col_widths[i] = col_widths[i].max(cell.len());
            }
        }

        rows.iter()
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
