//! Local spreadsheet access — Excel workbooks (`.xlsx`/`.xlsm`) and delimited text
//! (`.csv`/`.tsv`), behind one command surface.
//!
//! A capability service in the same family as `sheets` — the command surface deliberately
//! mirrors it (A1 ranges, `values get/update/clear`) so an agent that knows one can drive
//! the other. The only difference is where the data lives: this one reads and writes local
//! files rather than calling an HTTP API, so it takes no `ApiClient` and needs no profile
//! or credentials. That is incidental to what it is for, not a different kind of command,
//! so its output goes through the same response envelope as every other service.

use std::path::Path;

use serde_json::{json, Value};
use umya_spreadsheet::{reader, writer, Workbook};

use crate::{cli::*, error::AppError};

const SERVICE: &str = "excel";
/// Excel's last column, XFD.
const MAX_COLUMN: u32 = 16_384;

/// Which on-disk format a path refers to. Chosen by extension: the two are different
/// enough (a delimited file has one implicit sheet and no styling) that guessing from
/// content would only make behaviour harder to predict.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Backend {
    Xlsx,
    /// Character-delimited text — `.csv` (comma) and `.tsv`/`.tab` (tab).
    Delimited(u8),
    /// Formats we can read but not write: legacy `.xls`/`.xla` (BIFF), `.xlsb`, `.ods`.
    /// No maintained pure-Rust writer exists for these, and hand-rolling one risks
    /// emitting files Excel refuses to open, so writes are refused rather than attempted.
    ReadOnly,
}

impl Backend {
    fn for_path(file: &Path) -> Self {
        match file
            .extension()
            .and_then(|ext| ext.to_str())
            .map(str::to_ascii_lowercase)
            .as_deref()
        {
            Some("csv") => Self::Delimited(b','),
            Some("tsv") | Some("tab") => Self::Delimited(b'\t'),
            Some("xls") | Some("xla") | Some("xlsb") | Some("ods") => Self::ReadOnly,
            _ => Self::Xlsx,
        }
    }
}

/// A delimited file has exactly one sheet and no name of its own, so the file stem stands
/// in — that keeps range strings (`'sales'!A1:C3`) and responses shaped like the xlsx case.
fn implied_sheet_name(file: &Path) -> String {
    file.file_stem()
        .and_then(|stem| stem.to_str())
        .filter(|stem| !stem.is_empty())
        .unwrap_or("Sheet1")
        .to_string()
}

pub(crate) fn dispatch(command: ExcelCommand) -> Result<Value, AppError> {
    let value = match command.resource {
        ExcelResource::Workbook(cmd) => match cmd.action {
            ExcelWorkbookAction::Create(args) => {
                workbook_create(&args.file, args.sheets.as_deref(), args.force)
            }
        },
        ExcelResource::Sheets(cmd) => match cmd.action {
            ExcelSheetsAction::List(args) => sheets_list(&args.file),
        },
        ExcelResource::Values(cmd) => match cmd.action {
            ExcelValuesAction::Get(args) => values_get(&args.file, &args.range),
            ExcelValuesAction::Update(args) => {
                values_update(&args.file, &args.range, &args.values, args.force)
            }
            ExcelValuesAction::Clear(args) => values_clear(&args.file, &args.range, args.force),
        },
    }?;
    Ok(mark_complete(value))
}

/// Every excel response is the whole answer — a range read returns the entire rectangle and
/// nothing here pages. Say so explicitly, or the shared pagination envelope sees an array
/// (or the word "list") and tells the agent to go hunting for continuation parameters that
/// this service does not have.
fn mark_complete(value: Value) -> Value {
    match value {
        Value::Object(mut object) => {
            object.insert("truncated".to_string(), Value::Bool(false));
            Value::Object(object)
        }
        other => other,
    }
}

// ── commands ──────────────────────────────────────────────────────────────────

fn workbook_create(file: &Path, sheets: Option<&str>, force: bool) -> Result<Value, AppError> {
    let operation = "workbook.create";

    // Creating is the one destructive operation here, so an existing file is refused
    // unless the caller asked for the overwrite explicitly.
    if file.exists() && !force {
        return Err(AppError::invalid_input(
            SERVICE,
            operation,
            format!(
                "{} already exists; pass --force to overwrite it",
                file.display()
            ),
        ));
    }

    if Backend::for_path(file) == Backend::ReadOnly {
        return Err(read_only_is_write_blocked(operation, file));
    }

    if let Backend::Delimited(delimiter) = Backend::for_path(file) {
        // A delimited file is a single grid, so there is nothing for --sheets to name.
        // Reject it rather than accept a list and silently drop all but one.
        if sheets.is_some() {
            return Err(AppError::invalid_input(
                SERVICE,
                operation,
                "--sheets does not apply to a delimited file; it holds a single sheet named after the file",
            ));
        }
        write_grid(file, delimiter, &[], operation)?;
        return Ok(json!({
            "file": file.display().to_string(),
            "created": true,
            "sheets": [implied_sheet_name(file)],
        }));
    }

    let names = parse_sheet_names(sheets, operation)?;

    // new_file() seeds a "Sheet1"; build from an empty workbook instead so the requested
    // names are the only tabs, in the order given.
    let mut book = umya_spreadsheet::new_file_empty_worksheet();
    for name in &names {
        book.new_sheet(name).map_err(|err| {
            AppError::invalid_input(
                SERVICE,
                operation,
                format!("could not add sheet {name:?}: {err}"),
            )
        })?;
    }

    write(&book, file, operation)?;

    Ok(json!({
        "file": file.display().to_string(),
        "created": true,
        "sheets": names,
    }))
}

/// Split the `--sheets` list, defaulting to a single `Sheet1`. Excel rejects duplicate or
/// empty tab names, so catch those here with a clearer message than the writer would give.
fn parse_sheet_names(
    sheets: Option<&str>,
    operation: &'static str,
) -> Result<Vec<String>, AppError> {
    let Some(raw) = sheets else {
        return Ok(vec!["Sheet1".to_string()]);
    };
    let names: Vec<String> = raw
        .split(',')
        .map(|name| name.trim().to_string())
        .filter(|name| !name.is_empty())
        .collect();
    if names.is_empty() {
        return Err(AppError::invalid_input(
            SERVICE,
            operation,
            "--sheets listed no usable names",
        ));
    }
    for (i, name) in names.iter().enumerate() {
        if names[..i].contains(name) {
            return Err(AppError::invalid_input(
                SERVICE,
                operation,
                format!("duplicate sheet name {name:?}; tab names must be unique"),
            ));
        }
    }
    Ok(names)
}

fn sheets_list(file: &Path) -> Result<Value, AppError> {
    let operation = "sheets.list";
    if Backend::for_path(file) == Backend::ReadOnly {
        use calamine::Reader;
        let mut book = open_read_only(file, operation)?;
        let sheets: Vec<Value> = book
            .sheet_names()
            .into_iter()
            .enumerate()
            .map(|(index, name)| {
                let (cols, rows) = book
                    .worksheet_range(&name)
                    .ok()
                    .and_then(|range| range.end().map(|(r, c)| (c + 1, r + 1)))
                    .unwrap_or((0, 0));
                json!({
                    "index": index,
                    "title": name,
                    "usedRange": used_range_a1(&name, cols, rows),
                    "rowCount": rows,
                    "columnCount": cols,
                })
            })
            .collect();
        return Ok(json!({
            "file": file.display().to_string(),
            "readOnly": true,
            "sheets": sheets,
        }));
    }
    if let Backend::Delimited(delimiter) = Backend::for_path(file) {
        let grid = read_grid(file, delimiter, operation)?;
        let (cols, rows) = grid_extent(&grid);
        let title = implied_sheet_name(file);
        return Ok(json!({
            "file": file.display().to_string(),
            "sheets": [json!({
                "index": 0,
                "title": title,
                "usedRange": used_range_a1(&title, cols, rows),
                "rowCount": rows,
                "columnCount": cols,
            })],
        }));
    }
    let book = open(file, operation)?;
    let sheets: Vec<Value> = book
        .sheet_collection()
        .iter()
        .enumerate()
        .map(|(index, sheet)| {
            let (cols, rows) = sheet.highest_column_and_row();
            json!({
                "index": index,
                "title": sheet.name(),
                "usedRange": used_range_a1(sheet.name(), cols, rows),
                "rowCount": rows,
                "columnCount": cols,
            })
        })
        .collect();
    Ok(json!({ "file": file.display().to_string(), "sheets": sheets }))
}

fn values_get(file: &Path, range: &str) -> Result<Value, AppError> {
    let operation = "values.get";
    if Backend::for_path(file) == Backend::ReadOnly {
        use calamine::Reader;
        let mut book = open_read_only(file, operation)?;
        let names = book.sheet_names();
        let spec = RangeSpec::parse_in(range, operation, &|name| {
            names.iter().any(|known| known == name)
        })?;
        let title = match spec.sheet.as_deref() {
            Some(name) => names
                .iter()
                .find(|known| known.as_str() == name)
                .cloned()
                .ok_or_else(|| {
                    AppError::not_found(
                        SERVICE,
                        operation,
                        format!(
                            "no sheet named {name:?}; workbook has: {}",
                            names.join(", ")
                        ),
                    )
                })?,
            None => names.first().cloned().ok_or_else(|| {
                AppError::not_found(SERVICE, operation, "workbook has no sheets".to_string())
            })?,
        };
        let data = book.worksheet_range(&title).map_err(|err| {
            AppError::invalid_input(
                SERVICE,
                operation,
                format!("could not read sheet {title:?}: {err}"),
            )
        })?;

        // calamine reports the used range's own origin, and cell lookups are relative to
        // it, so convert 1-based A1 coordinates into that frame before indexing.
        let (origin_row, origin_col) = data.start().unwrap_or((0, 0));
        let (used_cols, used_rows) = data.end().map_or((0, 0), |(r, c)| (c + 1, r + 1));
        let rect = spec.resolve(used_cols, used_rows);

        let mut rows: Vec<Value> = Vec::new();
        for row in rect.start_row..=rect.end_row {
            let mut cells: Vec<Value> = Vec::new();
            for col in rect.start_col..=rect.end_col {
                let cell = (row - 1)
                    .checked_sub(origin_row)
                    .zip((col - 1).checked_sub(origin_col))
                    .and_then(|(r, c)| data.get((r as usize, c as usize)))
                    .map_or_else(|| Value::String(String::new()), read_only_value);
                cells.push(cell);
            }
            while matches!(cells.last(), Some(Value::String(s)) if s.is_empty()) {
                cells.pop();
            }
            rows.push(Value::Array(cells));
        }
        while matches!(rows.last(), Some(Value::Array(cells)) if cells.is_empty()) {
            rows.pop();
        }

        return Ok(json!({
            "file": file.display().to_string(),
            "range": rect.to_a1(&title),
            "majorDimension": "ROWS",
            "readOnly": true,
            "values": rows,
        }));
    }
    if let Backend::Delimited(delimiter) = Backend::for_path(file) {
        let grid = read_grid(file, delimiter, operation)?;
        let implied = implied_sheet_name(file);
        let spec = RangeSpec::parse_in(range, operation, &|name| name == implied)?;
        let title = check_delimited_sheet(&spec, file, operation)?;
        let (used_cols, used_rows) = grid_extent(&grid);
        let rect = spec.resolve(used_cols, used_rows);

        let mut rows: Vec<Value> = Vec::new();
        for row in rect.start_row..=rect.end_row {
            let line = grid.get((row - 1) as usize);
            let mut cells: Vec<Value> = Vec::new();
            for col in rect.start_col..=rect.end_col {
                let raw = line
                    .and_then(|line| line.get((col - 1) as usize))
                    .map(String::as_str)
                    .unwrap_or("");
                cells.push(delimited_value(raw));
            }
            while matches!(cells.last(), Some(Value::String(s)) if s.is_empty()) {
                cells.pop();
            }
            rows.push(Value::Array(cells));
        }
        while matches!(rows.last(), Some(Value::Array(cells)) if cells.is_empty()) {
            rows.pop();
        }

        return Ok(json!({
            "file": file.display().to_string(),
            "range": rect.to_a1(&title),
            "majorDimension": "ROWS",
            "values": rows,
        }));
    }
    let book = open(file, operation)?;
    let spec = RangeSpec::parse_in(range, operation, &|name| has_sheet(&book, name))?;
    let sheet = resolve_sheet(&book, spec.sheet.as_deref(), operation)?;
    let (used_cols, used_rows) = sheet.highest_column_and_row();
    let rect = spec.resolve(used_cols, used_rows);

    let mut rows: Vec<Value> = Vec::new();
    for row in rect.start_row..=rect.end_row {
        let mut cells: Vec<Value> = Vec::new();
        for col in rect.start_col..=rect.end_col {
            cells.push(cell_value(sheet, col, row));
        }
        // Google Sheets omits trailing empties rather than padding rows out to the
        // requested width; match that so callers see the same shape from both services.
        while matches!(cells.last(), Some(Value::String(s)) if s.is_empty()) {
            cells.pop();
        }
        rows.push(Value::Array(cells));
    }
    while matches!(rows.last(), Some(Value::Array(cells)) if cells.is_empty()) {
        rows.pop();
    }

    Ok(json!({
        "file": file.display().to_string(),
        "range": rect.to_a1(sheet.name()),
        "majorDimension": "ROWS",
        "values": rows,
    }))
}

fn values_update(
    file: &Path,
    range: &str,
    values_json: &str,
    force: bool,
) -> Result<Value, AppError> {
    let operation = "values.update";
    if Backend::for_path(file) == Backend::ReadOnly {
        return Err(read_only_is_write_blocked(operation, file));
    }
    let rows = parse_values(values_json, operation)?;
    if let Backend::Delimited(delimiter) = Backend::for_path(file) {
        let mut grid = read_grid(file, delimiter, operation)?;
        let implied = implied_sheet_name(file);
        let spec = RangeSpec::parse_in(range, operation, &|name| name == implied)?;
        let title = check_delimited_sheet(&spec, file, operation)?;

        let start_col = spec.start_col.unwrap_or(1);
        let start_row = spec.start_row.unwrap_or(1);
        let width = rows.iter().map(Vec::len).max().unwrap_or(0);

        let mut updated_cells = 0usize;
        for (row_offset, row) in rows.iter().enumerate() {
            for (col_offset, value) in row.iter().enumerate() {
                set_grid_cell(
                    &mut grid,
                    start_col + col_offset as u32,
                    start_row + row_offset as u32,
                    delimited_text(value),
                );
                updated_cells += 1;
            }
        }
        write_grid(file, delimiter, &grid, operation)?;

        let rect = Rect {
            start_col,
            start_row,
            end_col: start_col + width.saturating_sub(1) as u32,
            end_row: start_row + rows.len().saturating_sub(1) as u32,
        };
        return Ok(json!({
            "file": file.display().to_string(),
            "updatedRange": rect.to_a1(&title),
            "updatedRows": rows.len(),
            "updatedColumns": width,
            "updatedCells": updated_cells,
        }));
    }
    guard_rewrite(file, force, operation)?;
    let mut book = open(file, operation)?;
    let spec = RangeSpec::parse_in(range, operation, &|name| has_sheet(&book, name))?;
    let sheet_name = resolve_sheet_name(&book, spec.sheet.as_deref(), operation)?;

    // The anchor is the top-left of the range; the payload decides how far the write
    // extends, exactly like the Sheets API. A bounded range is not a constraint on size.
    let start_col = spec.start_col.unwrap_or(1);
    let start_row = spec.start_row.unwrap_or(1);
    let width = rows.iter().map(Vec::len).max().unwrap_or(0);

    let sheet = book
        .sheet_by_name_mut(&sheet_name)
        .map_err(|err| AppError::internal(SERVICE, operation, err.to_string()))?;

    let mut updated_cells = 0usize;
    for (row_offset, row) in rows.iter().enumerate() {
        for (col_offset, value) in row.iter().enumerate() {
            let col = start_col + col_offset as u32;
            let row_num = start_row + row_offset as u32;
            if value.is_null() {
                sheet.remove_cell((col, row_num));
            } else {
                set_cell(sheet, col, row_num, value);
            }
            updated_cells += 1;
        }
    }

    let rect = Rect {
        start_col,
        start_row,
        end_col: start_col + width.saturating_sub(1) as u32,
        end_row: start_row + rows.len().saturating_sub(1) as u32,
    };

    write(&book, file, operation)?;

    Ok(json!({
        "file": file.display().to_string(),
        "updatedRange": rect.to_a1(&sheet_name),
        "updatedRows": rows.len(),
        "updatedColumns": width,
        "updatedCells": updated_cells,
    }))
}

fn values_clear(file: &Path, range: &str, force: bool) -> Result<Value, AppError> {
    let operation = "values.clear";
    if Backend::for_path(file) == Backend::ReadOnly {
        return Err(read_only_is_write_blocked(operation, file));
    }
    if let Backend::Delimited(delimiter) = Backend::for_path(file) {
        let mut grid = read_grid(file, delimiter, operation)?;
        let implied = implied_sheet_name(file);
        let spec = RangeSpec::parse_in(range, operation, &|name| name == implied)?;
        let title = check_delimited_sheet(&spec, file, operation)?;
        let (used_cols, used_rows) = grid_extent(&grid);
        let rect = spec.resolve(used_cols, used_rows);

        let mut cleared = 0usize;
        for row in rect.start_row..=rect.end_row {
            let Some(line) = grid.get_mut((row - 1) as usize) else {
                continue;
            };
            for col in rect.start_col..=rect.end_col {
                if let Some(cell) = line.get_mut((col - 1) as usize) {
                    if !cell.is_empty() {
                        cell.clear();
                        cleared += 1;
                    }
                }
            }
        }
        write_grid(file, delimiter, &grid, operation)?;

        return Ok(json!({
            "file": file.display().to_string(),
            "clearedRange": rect.to_a1(&title),
            "clearedCells": cleared,
        }));
    }
    guard_rewrite(file, force, operation)?;
    let mut book = open(file, operation)?;
    let spec = RangeSpec::parse_in(range, operation, &|name| has_sheet(&book, name))?;
    let sheet_name = resolve_sheet_name(&book, spec.sheet.as_deref(), operation)?;

    let (used_cols, used_rows) = {
        let sheet = book
            .sheet_by_name(&sheet_name)
            .map_err(|err| AppError::internal(SERVICE, operation, err.to_string()))?;
        sheet.highest_column_and_row()
    };
    let rect = spec.resolve(used_cols, used_rows);

    let sheet = book
        .sheet_by_name_mut(&sheet_name)
        .map_err(|err| AppError::internal(SERVICE, operation, err.to_string()))?;
    let mut cleared = 0usize;
    for row in rect.start_row..=rect.end_row {
        for col in rect.start_col..=rect.end_col {
            if sheet.remove_cell((col, row)) {
                cleared += 1;
            }
        }
    }

    write(&book, file, operation)?;

    Ok(json!({
        "file": file.display().to_string(),
        "clearedRange": rect.to_a1(&sheet_name),
        "clearedCells": cleared,
    }))
}

// ── read-only backend (legacy .xls, .xlsb, .ods) ──────────────────────────────

/// Open a read-only workbook, turning an open failure into a diagnosis rather than a raw
/// parser error.
fn open_read_only(
    file: &Path,
    operation: &'static str,
) -> Result<calamine::Sheets<std::io::BufReader<std::fs::File>>, AppError> {
    if !file.exists() {
        return Err(AppError::not_found(
            SERVICE,
            operation,
            format!("no such file: {}", file.display()),
        ));
    }
    calamine::open_workbook_auto(file).map_err(|err| {
        AppError::invalid_input(
            SERVICE,
            operation,
            format!(
                "could not read {}: {}",
                file.display(),
                diagnose(file, &err)
            ),
        )
    })
}

/// Explain *why* a legacy file would not open.
///
/// Plenty of files named `.xls` are not BIFF at all — legacy "Export to Excel" buttons
/// routinely emit an HTML table or delimited text with a spreadsheet extension, and Excel
/// opens them anyway. Naming that case turns a baffling parser error into something the
/// caller can act on.
fn diagnose(file: &Path, err: &calamine::Error) -> String {
    use std::io::Read;

    let mut head = [0u8; 512];
    let read = std::fs::File::open(file)
        .and_then(|mut f| f.read(&mut head))
        .unwrap_or(0);
    let head = &head[..read];

    if head.starts_with(b"PK") {
        return "the file is a zip-based workbook despite its extension — rename it to .xlsx and retry".to_string();
    }
    if head.starts_with(&[0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1]) {
        // A real OLE2 container, so this is a genuine parse failure.
        return format!("{err}");
    }
    let text = String::from_utf8_lossy(head).to_ascii_lowercase();
    if text.contains("<html") || text.contains("<table") || text.contains("<?xml") {
        return "the file is really HTML with a spreadsheet extension (common from legacy \
                \"export to Excel\" buttons). Save it as .xlsx or .csv and retry"
            .to_string();
    }
    if !text.trim().is_empty() && (text.contains(',') || text.contains('\t')) {
        return "the file is really delimited text with a spreadsheet extension — rename it \
                to .csv (or .tsv) and retry"
            .to_string();
    }
    format!("{err}")
}

fn read_only_value(cell: &calamine::Data) -> Value {
    use calamine::Data;
    match cell {
        Data::Int(i) => Value::Number((*i).into()),
        Data::Float(f) => serde_json::Number::from_f64(*f)
            .map_or_else(|| Value::String(f.to_string()), Value::Number),
        Data::Bool(b) => Value::Bool(*b),
        Data::String(s) => Value::String(s.clone()),
        // Dates arrive as a serial number or an ISO string depending on how the file
        // stored them; render both as text so callers get something stable.
        Data::DateTime(dt) => Value::String(dt.to_string()),
        Data::DateTimeIso(s) | Data::DurationIso(s) => Value::String(s.clone()),
        Data::Error(e) => Value::String(format!("#ERROR:{e:?}")),
        Data::Empty => Value::String(String::new()),
    }
}

fn read_only_is_write_blocked(operation: &'static str, file: &Path) -> AppError {
    AppError::invalid_input(
        SERVICE,
        operation,
        format!(
            "{} is a read-only format for this tool; open it, then save as .xlsx (or .csv) to edit",
            file.display()
        ),
    )
}

// ── rewrite safety guard ──────────────────────────────────────────────────────

/// Workbook features that a read-modify-write cycle silently discards, keyed by the OOXML
/// part path that betrays their presence.
///
/// Editing an xlsx means parsing the whole workbook and writing a fresh one, so anything
/// the parser does not model is simply absent from the output. These are the categories a
/// sweep over umya's own fixture corpus proved to be dropped; cell values, styling and
/// `.xlsm` macros survive and are deliberately not listed.
const FRAGILE_PARTS: &[(&str, &str)] = &[
    ("xl/charts/", "chart"),
    ("xl/pivotTables/", "pivot table"),
    ("xl/pivotCache/", "pivot cache"),
    ("xl/ctrlProps/", "form control"),
    ("xl/externalLinks/", "external link"),
    ("customXml/", "custom XML part"),
    ("docMetadata/LabelInfo", "sensitivity label"),
    ("xl/persons/", "threaded comment author list"),
];

/// Count each at-risk feature in the workbook as it exists on disk.
///
/// Returns an empty vec for a plain data workbook, for a non-zip file, or for anything
/// unreadable — the guard's job is to stop confident destruction, not to second-guess
/// files the writer will reject on its own.
fn fragile_features(file: &Path) -> Vec<(String, usize)> {
    let Ok(handle) = std::fs::File::open(file) else {
        return Vec::new();
    };
    let Ok(archive) = zip::ZipArchive::new(std::io::BufReader::new(handle)) else {
        return Vec::new();
    };

    let names: Vec<String> = archive.file_names().map(str::to_string).collect();
    let mut found: Vec<(String, usize)> = Vec::new();
    for (prefix, label) in FRAGILE_PARTS {
        // Count only the parts themselves, not their _rels companions, so "2 charts"
        // means two charts rather than two charts plus their relationship files.
        let count = names
            .iter()
            .filter(|name| name.starts_with(prefix) && !name.contains("/_rels/"))
            .count();
        if count > 0 {
            found.push(((*label).to_string(), count));
        }
    }
    // vmlDrawing backs form controls and legacy comments; plain images live in
    // drawing*.xml, which umya does round-trip, so only flag the vml case.
    let vml = names.iter().filter(|n| n.contains("vmlDrawing")).count();
    if vml > 0 {
        found.push(("legacy drawing/comment layer".to_string(), vml));
    }
    found
}

fn describe_features(found: &[(String, usize)]) -> String {
    found
        .iter()
        .map(|(label, count)| {
            if *count == 1 {
                format!("1 {label}")
            } else {
                format!("{count} {label}s")
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// Refuse to rewrite a workbook whose features the write would destroy.
fn guard_rewrite(file: &Path, force: bool, operation: &'static str) -> Result<(), AppError> {
    if force {
        return Ok(());
    }
    let found = fragile_features(file);
    if found.is_empty() {
        return Ok(());
    }
    Err(AppError::invalid_input(
        SERVICE,
        operation,
        format!(
            "refusing to write {}: saving rewrites the whole workbook and would drop {}. \
             Pass --force to write anyway, or copy the values into a new file instead.",
            file.display(),
            describe_features(&found)
        ),
    ))
}

// ── delimited (csv/tsv) backend ───────────────────────────────────────────────

/// Read the whole file into a row-major grid of raw strings.
///
/// Rows are not padded to a common width here — callers index defensively — because a
/// ragged csv is normal and padding would invent trailing empty cells that then show up
/// in `usedRange`.
fn read_grid(
    file: &Path,
    delimiter: u8,
    operation: &'static str,
) -> Result<Vec<Vec<String>>, AppError> {
    if !file.exists() {
        return Err(AppError::not_found(
            SERVICE,
            operation,
            format!("no such file: {}", file.display()),
        ));
    }
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(delimiter)
        // Every row is data; the caller decides whether row 1 is a header.
        .has_headers(false)
        .flexible(true)
        .from_path(file)
        .map_err(|err| {
            AppError::invalid_input(
                SERVICE,
                operation,
                format!("could not read {}: {err}", file.display()),
            )
        })?;

    let mut grid = Vec::new();
    for record in reader.records() {
        let record = record.map_err(|err| {
            AppError::invalid_input(
                SERVICE,
                operation,
                format!("malformed delimited data in {}: {err}", file.display()),
            )
        })?;
        grid.push(record.iter().map(str::to_string).collect());
    }
    Ok(grid)
}

fn write_grid(
    file: &Path,
    delimiter: u8,
    grid: &[Vec<String>],
    operation: &'static str,
) -> Result<(), AppError> {
    // Write to a sibling temp file and rename, so an interrupted write can't leave the
    // caller with a half-truncated file. Mirrors what the xlsx writer does internally.
    let tmp = file.with_extension(format!(
        "{}tmp",
        file.extension().and_then(|e| e.to_str()).unwrap_or("csv")
    ));
    let mut writer = csv::WriterBuilder::new()
        .delimiter(delimiter)
        .flexible(true)
        .from_path(&tmp)
        .map_err(|err| {
            AppError::internal(
                SERVICE,
                operation,
                format!("could not write {}: {err}", tmp.display()),
            )
        })?;
    for row in grid {
        writer.write_record(row).map_err(|err| {
            AppError::internal(SERVICE, operation, format!("could not write a row: {err}"))
        })?;
    }
    writer.flush().map_err(|err| {
        AppError::internal(SERVICE, operation, format!("could not flush output: {err}"))
    })?;
    drop(writer);
    std::fs::rename(&tmp, file).map_err(|err| {
        let _ = std::fs::remove_file(&tmp);
        AppError::internal(
            SERVICE,
            operation,
            format!("could not replace {}: {err}", file.display()),
        )
    })
}

fn grid_extent(grid: &[Vec<String>]) -> (u32, u32) {
    let rows = grid.len() as u32;
    let cols = grid.iter().map(Vec::len).max().unwrap_or(0) as u32;
    (cols, rows)
}

/// Delimited files carry no types, so infer — but only when the parsed value renders back
/// to exactly the original text. That keeps `007`, `1.50`, `+3` and ` 1` as strings
/// instead of silently rewriting a zip code or a part number as a number.
fn delimited_value(raw: &str) -> Value {
    if raw.is_empty() {
        return Value::String(String::new());
    }
    if let Ok(n) = raw.parse::<i64>() {
        if n.to_string() == raw {
            return Value::Number(n.into());
        }
    }
    if let Ok(f) = raw.parse::<f64>() {
        if let Some(number) = serde_json::Number::from_f64(f) {
            if number.to_string() == raw {
                return Value::Number(number);
            }
        }
    }
    match raw {
        "true" | "TRUE" => return Value::Bool(true),
        "false" | "FALSE" => return Value::Bool(false),
        _ => {}
    }
    Value::String(raw.to_string())
}

/// Render a JSON value back to cell text. Strings pass through untouched so a round trip
/// through `values get` -> `values update` is lossless.
fn delimited_text(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::String(s) => s.clone(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        other => other.to_string(),
    }
}

/// Confirm a range's sheet component (if any) names this file's single implicit sheet.
fn check_delimited_sheet(
    spec: &RangeSpec,
    file: &Path,
    operation: &'static str,
) -> Result<String, AppError> {
    let implied = implied_sheet_name(file);
    match spec.sheet.as_deref() {
        Some(name) if name != implied => Err(AppError::not_found(
            SERVICE,
            operation,
            format!(
                "no sheet named {name:?}; {} is a delimited file with a single sheet: {implied}",
                file.display()
            ),
        )),
        _ => Ok(implied),
    }
}

fn set_grid_cell(grid: &mut Vec<Vec<String>>, col: u32, row: u32, text: String) {
    let row_idx = (row - 1) as usize;
    let col_idx = (col - 1) as usize;
    if grid.len() <= row_idx {
        grid.resize(row_idx + 1, Vec::new());
    }
    let line = &mut grid[row_idx];
    if line.len() <= col_idx {
        line.resize(col_idx + 1, String::new());
    }
    line[col_idx] = text;
}

// ── workbook helpers ──────────────────────────────────────────────────────────

fn open(file: &Path, operation: &'static str) -> Result<Workbook, AppError> {
    if !file.exists() {
        return Err(AppError::not_found(
            SERVICE,
            operation,
            format!("no such file: {}", file.display()),
        ));
    }
    reader::xlsx::read(file).map_err(|err| {
        AppError::invalid_input(
            SERVICE,
            operation,
            format!(
                "could not read {} as an .xlsx workbook: {err}",
                file.display()
            ),
        )
    })
}

fn write(book: &Workbook, file: &Path, operation: &'static str) -> Result<(), AppError> {
    writer::xlsx::write(book, file).map_err(|err| {
        AppError::internal(
            SERVICE,
            operation,
            format!("could not write {}: {err}", file.display()),
        )
    })
}

fn resolve_sheet<'a>(
    book: &'a Workbook,
    name: Option<&str>,
    operation: &'static str,
) -> Result<&'a umya_spreadsheet::Worksheet, AppError> {
    match name {
        Some(name) => book
            .sheet_by_name(name)
            .map_err(|_| unknown_sheet(book, name, operation)),
        None => book.sheet(0).map_err(|_| {
            AppError::not_found(SERVICE, operation, "workbook has no sheets".to_string())
        }),
    }
}

fn resolve_sheet_name(
    book: &Workbook,
    name: Option<&str>,
    operation: &'static str,
) -> Result<String, AppError> {
    Ok(resolve_sheet(book, name, operation)?.name().to_string())
}

fn has_sheet(book: &Workbook, name: &str) -> bool {
    book.sheet_collection().iter().any(|s| s.name() == name)
}

fn unknown_sheet(book: &Workbook, name: &str, operation: &'static str) -> AppError {
    let available: Vec<&str> = book.sheet_collection().iter().map(|s| s.name()).collect();
    AppError::not_found(
        SERVICE,
        operation,
        format!(
            "no sheet named {name:?}; workbook has: {}",
            available.join(", ")
        ),
    )
}

/// Read one cell as typed JSON — numbers stay numbers, so callers don't have to reparse
/// what Excel already knows the type of.
fn cell_value(sheet: &umya_spreadsheet::Worksheet, col: u32, row: u32) -> Value {
    let Some(cell) = sheet.cell((col, row)) else {
        return Value::String(String::new());
    };
    let raw = cell.value();
    match cell.data_type() {
        "n" => raw
            .parse::<f64>()
            .ok()
            .and_then(serde_json::Number::from_f64)
            .map_or_else(|| Value::String(raw.to_string()), Value::Number),
        "b" => Value::Bool(raw == "TRUE" || raw == "true" || raw == "1"),
        _ => Value::String(raw.to_string()),
    }
}

fn set_cell(sheet: &mut umya_spreadsheet::Worksheet, col: u32, row: u32, value: &Value) {
    let cell = sheet.cell_mut((col, row));
    match value {
        Value::Number(n) => {
            if let Some(f) = n.as_f64() {
                cell.set_value_number(f);
            } else {
                cell.set_value(n.to_string());
            }
        }
        Value::Bool(b) => {
            cell.set_value_bool(*b);
        }
        Value::String(s) => {
            // set_value re-guesses the type from the text, which would turn the string
            // "007" into the number 7 and drop a SKU's leading zeros. The caller already
            // said this is a string by sending a JSON string, so store it as one.
            cell.set_value_string(s.clone());
        }
        // Arrays/objects have no cell representation; store their JSON text rather than
        // silently dropping the caller's data.
        other => {
            cell.set_value(other.to_string());
        }
    }
}

fn parse_values(raw: &str, operation: &'static str) -> Result<Vec<Vec<Value>>, AppError> {
    let parsed: Value = serde_json::from_str(raw).map_err(|err| {
        AppError::invalid_input(
            SERVICE,
            operation,
            format!("--values is not valid JSON: {err}"),
        )
    })?;
    let Value::Array(rows) = parsed else {
        return Err(AppError::invalid_input(
            SERVICE,
            operation,
            "--values must be a JSON array of row arrays, e.g. [[\"a\",1],[\"b\",2]]",
        ));
    };
    rows.into_iter()
        .map(|row| match row {
            Value::Array(cells) => Ok(cells),
            _ => Err(AppError::invalid_input(
                SERVICE,
                operation,
                "each element of --values must itself be an array (one per row)",
            )),
        })
        .collect()
}

// ── A1 notation ───────────────────────────────────────────────────────────────

/// A parsed A1 range. Every bound is optional so open forms (`A:D`, `2:5`, a bare sheet
/// name) survive parsing and get resolved against the sheet's used range later.
#[derive(Debug, PartialEq)]
struct RangeSpec {
    sheet: Option<String>,
    start_col: Option<u32>,
    start_row: Option<u32>,
    end_col: Option<u32>,
    end_row: Option<u32>,
}

#[derive(Debug, PartialEq)]
struct Rect {
    start_col: u32,
    start_row: u32,
    end_col: u32,
    end_row: u32,
}

impl RangeSpec {
    /// Parse without a workbook to consult. Only for cases where the ambiguity below
    /// cannot arise (and for unit tests) — prefer [`RangeSpec::parse_in`].
    #[cfg(test)]
    fn parse(range: &str, operation: &'static str) -> Result<Self, AppError> {
        Self::parse_in(range, operation, &|_| false)
    }

    /// Parse an A1 range, using `sheet_exists` to settle the genuinely ambiguous case.
    ///
    /// A bare token like `Sheet1` is both a valid cell reference (column "SHEET", row 1)
    /// and by far the most common worksheet name there is. Excel resolves this from
    /// context and so do we: if the workbook has a tab by that name, it's a tab.
    fn parse_in(
        range: &str,
        operation: &'static str,
        sheet_exists: &dyn Fn(&str) -> bool,
    ) -> Result<Self, AppError> {
        let trimmed = range.trim();
        if trimmed.is_empty() {
            return Err(AppError::invalid_input(
                SERVICE,
                operation,
                "range is empty",
            ));
        }

        // Split the optional `Sheet!` prefix. Quoted names may themselves contain '!',
        // so honour the closing quote before looking for the separator.
        let (sheet, cells) = split_sheet_prefix(trimmed, sheet_exists);

        if cells.is_empty() {
            // A bare sheet name — the whole used range.
            return Ok(Self {
                sheet,
                start_col: None,
                start_row: None,
                end_col: None,
                end_row: None,
            });
        }

        let (start, end) = match cells.split_once(':') {
            Some((a, b)) => (a, Some(b)),
            None => (cells, None),
        };
        let (start_col, start_row) = parse_cell(start, operation)?;
        let (end_col, end_row) = match end {
            Some(end) => parse_cell(end, operation)?,
            // A single cell is a 1x1 range.
            None => (start_col, start_row),
        };

        Ok(Self {
            sheet,
            start_col,
            start_row,
            end_col,
            end_row,
        })
    }

    /// Fill in whatever the range left open using the sheet's used extent, and normalise
    /// inverted bounds (`D5:A1` means the same rectangle as `A1:D5`).
    fn resolve(&self, used_cols: u32, used_rows: u32) -> Rect {
        let start_col = self.start_col.unwrap_or(1);
        let start_row = self.start_row.unwrap_or(1);
        let end_col = self.end_col.unwrap_or_else(|| used_cols.max(start_col));
        let end_row = self.end_row.unwrap_or_else(|| used_rows.max(start_row));
        Rect {
            start_col: start_col.min(end_col),
            start_row: start_row.min(end_row),
            end_col: start_col.max(end_col),
            end_row: start_row.max(end_row),
        }
    }
}

impl Rect {
    fn to_a1(&self, sheet: &str) -> String {
        format!(
            "'{}'!{}{}:{}{}",
            // Double any apostrophe so the emitted range parses back to this same sheet —
            // callers are expected to feed `range`/`updatedRange` straight into the next call.
            sheet.replace('\'', "''"),
            col_to_letters(self.start_col),
            self.start_row,
            col_to_letters(self.end_col),
            self.end_row
        )
    }
}

fn used_range_a1(sheet: &str, cols: u32, rows: u32) -> String {
    Rect {
        start_col: 1,
        start_row: 1,
        end_col: cols.max(1),
        end_row: rows.max(1),
    }
    .to_a1(sheet)
}

fn split_sheet_prefix<'a>(
    range: &'a str,
    sheet_exists: &dyn Fn(&str) -> bool,
) -> (Option<String>, &'a str) {
    if range.starts_with('\'') {
        // Inside a quoted name a literal apostrophe is written doubled, so only a lone
        // quote closes the name. Without this, a tab called `Sh!eet'4` would terminate
        // early and the rest of its name would be parsed as cells.
        let bytes = range.as_bytes();
        let mut name = String::new();
        let mut i = 1;
        while i < bytes.len() {
            if bytes[i] == b'\'' {
                if bytes.get(i + 1) == Some(&b'\'') {
                    name.push('\'');
                    i += 2;
                    continue;
                }
                let after = &range[i + 1..];
                return (Some(name), after.strip_prefix('!').unwrap_or(after));
            }
            let ch = range[i..].chars().next().unwrap_or('\0');
            name.push(ch);
            i += ch.len_utf8();
        }
        // Unterminated quote — treat the whole thing as cells and let parsing complain.
        return (None, range);
    }
    match range.split_once('!') {
        Some((name, cells)) => (Some(name.to_string()), cells),
        None => {
            // No '!' at all: either a bare cell/range, or a bare (unquoted) sheet name.
            // An existing tab wins over a cell reading, so `Sheet1` means the tab.
            if sheet_exists(range) || !looks_like_cells(range) {
                (Some(range.to_string()), "")
            } else {
                (None, range)
            }
        }
    }
}

/// True when the text is A1-ish (`A1`, `A1:D5`, `A:D`, `2:5`) rather than a sheet name.
///
/// A ':' settles it — Excel forbids it in sheet names — so anything containing one is
/// treated as a range and any malformed endpoint surfaces as a parse error rather than
/// being mistaken for a tab. Without a ':' only a full `<letters><digits>` cell counts;
/// bare letters like `Inventory` (or even `A`) are far likelier to be a sheet name.
fn looks_like_cells(text: &str) -> bool {
    if text.contains(':') {
        return true;
    }
    let letters = text.chars().take_while(char::is_ascii_alphabetic).count();
    let digits = &text[letters..];
    // No column runs past XFD, so a longer alphabetic prefix can only be a sheet name —
    // which yields "no sheet named ..." instead of a baffling column-out-of-range error.
    letters > 0 && letters <= 3 && !digits.is_empty() && digits.chars().all(|c| c.is_ascii_digit())
}

/// Parse one A1 endpoint into (column, row), either of which may be absent for the open
/// forms: `A` has no row, `5` has no column.
fn parse_cell(text: &str, operation: &'static str) -> Result<(Option<u32>, Option<u32>), AppError> {
    let text = text.trim();
    if text.is_empty() {
        return Err(AppError::invalid_input(
            SERVICE,
            operation,
            "range endpoint is empty; use A1 notation such as A1:D5",
        ));
    }
    let letters: String = text.chars().take_while(char::is_ascii_alphabetic).collect();
    let digits = &text[letters.len()..];

    if !digits.chars().all(|c| c.is_ascii_digit()) {
        return Err(AppError::invalid_input(
            SERVICE,
            operation,
            format!(
                "{text:?} is not valid A1 notation; expected something like A1, A1:D5, A:D or 2:5"
            ),
        ));
    }

    let col = if letters.is_empty() {
        None
    } else {
        let col = letters_to_col(&letters);
        // Excel's last column is XFD (16384). Rejecting beyond it also keeps
        // letters_to_col from overflowing on a long alphabetic run.
        if !(1..=MAX_COLUMN).contains(&col) {
            return Err(AppError::invalid_input(
                SERVICE,
                operation,
                format!("column {letters:?} is beyond Excel's last column (XFD)"),
            ));
        }
        Some(col)
    };
    let row = if digits.is_empty() {
        None
    } else {
        Some(digits.parse::<u32>().map_err(|_| {
            AppError::invalid_input(
                SERVICE,
                operation,
                format!("row number out of range in {text:?}"),
            )
        })?)
    };

    if row == Some(0) {
        return Err(AppError::invalid_input(
            SERVICE,
            operation,
            "row numbers are 1-based; row 0 does not exist",
        ));
    }
    Ok((col, row))
}

fn letters_to_col(letters: &str) -> u32 {
    letters.bytes().fold(0u32, |acc, b| {
        acc.saturating_mul(26)
            .saturating_add(u32::from(b.to_ascii_uppercase() - b'A') + 1)
    })
}

fn col_to_letters(mut col: u32) -> String {
    let mut out = Vec::new();
    while col > 0 {
        let rem = (col - 1) % 26;
        out.push(b'A' + rem as u8);
        col = (col - 1) / 26;
    }
    out.reverse();
    String::from_utf8(out).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    const OP: &str = "values.get";

    // ── file-backed tests ─────────────────────────────────────────────────────
    //
    // These drive the real commands against real files on disk. The parser tests below
    // cover A1 handling in isolation; these cover the round trips that actually matter.

    fn temp_dir() -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("aai-excel-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    fn write_csv(name: &str, body: &str) -> std::path::PathBuf {
        let path = temp_dir().join(name);
        std::fs::write(&path, body).expect("write fixture");
        path
    }

    fn values_of(response: &Value) -> &Vec<Value> {
        response["values"].as_array().expect("values array")
    }

    fn fixture(name: &str) -> std::path::PathBuf {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures")
            .join(name)
    }

    /// Build a workbook-shaped zip containing the given part names, so the guard can be
    /// tested without depending on a particular real-world file.
    fn zip_with_parts(name: &str, parts: &[&str]) -> std::path::PathBuf {
        use std::io::Write;
        let path = temp_dir().join(name);
        let file = std::fs::File::create(&path).expect("create zip");
        let mut writer = zip::ZipWriter::new(file);
        let opts: zip::write::SimpleFileOptions = Default::default();
        for part in parts {
            writer.start_file(*part, opts).expect("start part");
            writer.write_all(b"<xml/>").expect("write part");
        }
        writer.finish().expect("finish zip");
        path
    }

    // ── rewrite guard ─────────────────────────────────────────────────────────

    #[test]
    fn guard_allows_a_plain_workbook() {
        let path = zip_with_parts(
            "plain.xlsx",
            &["xl/workbook.xml", "xl/worksheets/sheet1.xml"],
        );
        assert!(fragile_features(&path).is_empty());
        assert!(guard_rewrite(&path, false, "values.update").is_ok());
    }

    #[test]
    fn guard_blocks_a_workbook_whose_features_a_rewrite_would_drop() {
        let path = zip_with_parts(
            "rich.xlsx",
            &[
                "xl/workbook.xml",
                "xl/charts/chart1.xml",
                "xl/charts/chart2.xml",
                "xl/charts/_rels/chart1.xml.rels",
                "xl/pivotTables/pivotTable1.xml",
            ],
        );
        let found = fragile_features(&path);
        // _rels companions must not inflate the count — two charts, not three.
        assert_eq!(
            found.iter().find(|(l, _)| l == "chart").map(|(_, n)| *n),
            Some(2)
        );

        let err = guard_rewrite(&path, false, "values.update").expect_err("must refuse");
        assert_eq!(err.code, "invalid_input");
        assert!(err.message.contains("2 charts"), "{}", err.message);
        assert!(err.message.contains("1 pivot table"), "{}", err.message);
        assert!(err.message.contains("--force"), "{}", err.message);

        // --force is the documented escape hatch.
        assert!(guard_rewrite(&path, true, "values.update").is_ok());
    }

    #[test]
    fn guard_ignores_files_it_cannot_inspect() {
        // A non-zip must not trip the guard; the writer will reject it on its own terms.
        let path = write_csv("notazip.xlsx", "not a workbook");
        assert!(guard_rewrite(&path, false, "values.update").is_ok());
    }

    #[test]
    fn guard_pluralises_counts() {
        assert_eq!(
            describe_features(&[("chart".into(), 1), ("pivot table".into(), 3)]),
            "1 chart, 3 pivot tables"
        );
    }

    // ── read-only formats ─────────────────────────────────────────────────────

    #[test]
    fn backend_is_chosen_by_extension() {
        use std::path::Path as P;
        assert_eq!(Backend::for_path(P::new("a.xlsx")), Backend::Xlsx);
        assert_eq!(Backend::for_path(P::new("a.XLSM")), Backend::Xlsx);
        assert_eq!(Backend::for_path(P::new("a.csv")), Backend::Delimited(b','));
        assert_eq!(
            Backend::for_path(P::new("a.tsv")),
            Backend::Delimited(b'\t')
        );
        for legacy in ["a.xls", "a.xla", "a.xlsb", "a.ods"] {
            assert_eq!(
                Backend::for_path(P::new(legacy)),
                Backend::ReadOnly,
                "{legacy}"
            );
        }
    }

    #[test]
    fn legacy_xls_is_readable() {
        let path = fixture("legacy.xls");
        let listed = sheets_list(&path).expect("list legacy xls");
        assert_eq!(listed["readOnly"], json!(true));
        assert_eq!(listed["sheets"][0]["title"], json!("Summary"));
        assert_eq!(listed["sheets"][1]["title"], json!("Notes"));

        let out = values_get(&path, "Summary!A1:C3").expect("read legacy xls");
        let rows = values_of(&out);
        assert_eq!(rows[0], json!(["Item", "Cost", "Stocked"]));
        assert_eq!(rows[1], json!(["Wheel", 20.5, true]));
        assert_eq!(rows[2], json!(["Door", 15, false]));
    }

    #[test]
    fn read_only_formats_refuse_every_write() {
        let path = fixture("legacy.xls");
        for err in [
            values_update(&path, "A1", r#"[["x"]]"#, false).expect_err("update"),
            values_clear(&path, "A1", false).expect_err("clear"),
            // --force must not unlock a format we simply cannot write.
            values_update(&path, "A1", r#"[["x"]]"#, true).expect_err("forced update"),
        ] {
            assert_eq!(err.code, "invalid_input");
            assert!(err.message.contains("read-only"), "{}", err.message);
        }
    }

    #[test]
    fn masquerading_files_are_diagnosed_not_just_rejected() {
        // Legacy "export to Excel" buttons emit these constantly; a generic parse error
        // would leave the caller with no idea what to do next.
        let cases = [
            (
                "html.xls",
                "<html><body><table><tr><td>a</td></tr></table>",
                "HTML",
            ),
            ("text.xls", "name,qty\nbolt,3\n", "delimited text"),
        ];
        for (name, body, expected) in cases {
            let path = write_csv(name, body);
            let err = values_get(&path, "A1").expect_err("should not parse");
            assert_eq!(err.code, "invalid_input");
            assert!(err.message.contains(expected), "{name}: {}", err.message);
        }
    }

    #[test]
    fn csv_reads_a_range_with_inferred_types() {
        let path = write_csv(
            "read.csv",
            "Item,Cost,Stocked\nWheel,20.5,true\nDoor,15,false\n",
        );
        let out = values_get(&path, "A1:C3").expect("read csv");

        let rows = values_of(&out);
        assert_eq!(rows[0], json!(["Item", "Cost", "Stocked"]));
        assert_eq!(rows[1], json!(["Wheel", 20.5, true]));
        assert_eq!(rows[2], json!(["Door", 15, false]));
        assert_eq!(out["range"], json!("'read'!A1:C3"));
    }

    #[test]
    fn xlsx_write_preserves_strings_that_look_numeric() {
        // Regression: umya's set_value re-guesses the type from the text, so writing the
        // JSON string "007" stored the number 7 and lost the leading zeros. A JSON string
        // is the caller stating the type; honour it.
        let path = temp_dir().join("typed.xlsx");
        let _ = std::fs::remove_file(&path);
        workbook_create(&path, Some("Data"), false).expect("create");
        values_update(
            &path,
            "Data!A1",
            r#"[["007","1.50","0x10",42,20.5,true]]"#,
            false,
        )
        .expect("write");

        let read = values_get(&path, "Data!A1:F1").expect("read");
        let row = &values_of(&read)[0];
        assert_eq!(row[0], json!("007"), "leading zeros must survive");
        assert_eq!(row[1], json!("1.50"), "trailing zero must survive");
        assert_eq!(row[2], json!("0x10"));
        // Genuine JSON numbers and bools still round-trip as numbers and bools.
        assert_eq!(row[3], json!(42.0));
        assert_eq!(row[4], json!(20.5));
        assert_eq!(row[5], json!(true));
    }

    #[test]
    fn csv_preserves_text_that_only_looks_numeric() {
        // Zip codes, part numbers and padded decimals must survive a read/write cycle as
        // text — inferring them as numbers would silently rewrite the user's data.
        let path = write_csv("textish.csv", "007,1.50,+3,0x10, 42\n");
        let rows = values_get(&path, "A1:E1").expect("read csv");
        assert_eq!(
            values_of(&rows)[0],
            json!(["007", "1.50", "+3", "0x10", " 42"])
        );
    }

    #[test]
    fn csv_round_trips_quoted_fields() {
        // Embedded delimiters, quotes and newlines are the classic csv trap.
        let path = write_csv(
            "quoted.csv",
            "name,note\n\"Smith, Jo\",\"said \"\"hi\"\"\"\n\"two\nlines\",ok\n",
        );
        let before = values_get(&path, "A1:B3").expect("read");
        assert_eq!(values_of(&before)[1], json!(["Smith, Jo", "said \"hi\""]));
        assert_eq!(values_of(&before)[2], json!(["two\nlines", "ok"]));

        // Rewrite an unrelated cell, then confirm the tricky fields are untouched.
        values_update(&path, "D1", r#"[["x"]]"#, false).expect("update");
        let after = values_get(&path, "A1:B3").expect("re-read");
        assert_eq!(values_of(&after)[1], json!(["Smith, Jo", "said \"hi\""]));
        assert_eq!(values_of(&after)[2], json!(["two\nlines", "ok"]));
    }

    #[test]
    fn csv_update_expands_the_grid_and_clear_blanks_cells() {
        let path = write_csv("edit.csv", "a,b\nc,d\n");
        let updated = values_update(&path, "B3", r#"[["x","y"],["z",1]]"#, false).expect("update");
        assert_eq!(updated["updatedRange"], json!("'edit'!B3:C4"));
        assert_eq!(updated["updatedCells"], json!(4));

        let out = values_get(&path, "A1:C4").expect("read back");
        assert_eq!(values_of(&out)[2], json!(["", "x", "y"]));
        assert_eq!(values_of(&out)[3], json!(["", "z", 1]));

        let cleared = values_clear(&path, "B3:C4", false).expect("clear");
        assert_eq!(cleared["clearedCells"], json!(4));
        let out = values_get(&path, "A1:C4").expect("read after clear");
        // Trailing empties are trimmed, so those rows collapse to nothing.
        assert_eq!(values_of(&out).len(), 2);
    }

    #[test]
    fn csv_sheet_name_comes_from_the_file_stem() {
        let path = write_csv("sales.csv", "a\n");
        let listed = sheets_list(&path).expect("list");
        assert_eq!(listed["sheets"][0]["title"], json!("sales"));

        // The implicit sheet may be named explicitly...
        assert!(values_get(&path, "'sales'!A1").is_ok());
        // ...but a different name is an error, not a silent fallback.
        let err = values_get(&path, "'Sheet1'!A1").expect_err("wrong sheet must fail");
        assert_eq!(err.code, "not_found");
    }

    #[test]
    fn tsv_is_tab_delimited() {
        let path = write_csv("tabbed.tsv", "a\tb\nc\td\n");
        let out = values_get(&path, "A1:B2").expect("read tsv");
        assert_eq!(values_of(&out)[0], json!(["a", "b"]));
    }

    #[test]
    fn csv_ragged_rows_are_tolerated() {
        let path = write_csv("ragged.csv", "a,b,c\nd\ne,f\n");
        let out = values_get(&path, "A1:C3").expect("read ragged");
        assert_eq!(values_of(&out)[1], json!(["d"]));
        assert_eq!(values_of(&out)[2], json!(["e", "f"]));
    }

    #[test]
    fn csv_create_rejects_sheet_names_and_refuses_to_clobber() {
        let path = temp_dir().join("fresh.csv");
        let _ = std::fs::remove_file(&path);

        assert!(workbook_create(&path, Some("A,B"), false).is_err());
        workbook_create(&path, None, false).expect("create csv");
        assert!(path.exists());
        // Second create without --force must not overwrite.
        assert!(workbook_create(&path, None, false).is_err());
        workbook_create(&path, None, true).expect("force overwrite");
    }

    #[test]
    fn missing_file_is_not_found_and_junk_is_invalid_input() {
        let missing = temp_dir().join("nope.csv");
        let _ = std::fs::remove_file(&missing);
        assert_eq!(
            values_get(&missing, "A1").expect_err("missing").code,
            "not_found"
        );

        // A non-xlsx handed to the xlsx backend must fail cleanly, not panic. This is the
        // legacy .xls / wrong-format path.
        let junk = write_csv(
            "legacy.xls",
            "\u{d0}\u{cf}\u{11}\u{e0}not really a workbook",
        );
        assert_eq!(
            values_get(&junk, "A1").expect_err("junk").code,
            "invalid_input"
        );
    }

    fn spec(range: &str) -> RangeSpec {
        RangeSpec::parse(range, OP).expect("range should parse")
    }

    #[test]
    fn column_letters_round_trip() {
        for (col, letters) in [(1, "A"), (26, "Z"), (27, "AA"), (52, "AZ"), (703, "AAA")] {
            assert_eq!(col_to_letters(col), letters);
            assert_eq!(letters_to_col(letters), col);
        }
    }

    #[test]
    fn parses_quoted_sheet_with_bang_in_name() {
        let parsed = spec("'Q1!Final'!A1:B2");
        assert_eq!(parsed.sheet.as_deref(), Some("Q1!Final"));
        assert_eq!(parsed.start_col, Some(1));
        assert_eq!(parsed.end_row, Some(2));
    }

    #[test]
    fn parses_unquoted_sheet_prefix() {
        let parsed = spec("Sheet1!B3:D9");
        assert_eq!(parsed.sheet.as_deref(), Some("Sheet1"));
        assert_eq!(parsed.start_col, Some(2));
        assert_eq!(parsed.start_row, Some(3));
        assert_eq!(parsed.end_col, Some(4));
        assert_eq!(parsed.end_row, Some(9));
    }

    #[test]
    fn bare_range_has_no_sheet() {
        let parsed = spec("A1:D5");
        assert_eq!(parsed.sheet, None);
        assert_eq!(parsed.start_col, Some(1));
        assert_eq!(parsed.end_col, Some(4));
    }

    #[test]
    fn bare_sheet_name_selects_whole_used_range() {
        let parsed = spec("Inventory");
        assert_eq!(parsed.sheet.as_deref(), Some("Inventory"));
        assert_eq!(parsed.start_col, None);
        assert_eq!(
            parsed.resolve(4, 10),
            Rect {
                start_col: 1,
                start_row: 1,
                end_col: 4,
                end_row: 10
            }
        );
    }

    #[test]
    fn single_cell_is_a_one_by_one_rect() {
        assert_eq!(
            spec("C7").resolve(99, 99),
            Rect {
                start_col: 3,
                start_row: 7,
                end_col: 3,
                end_row: 7
            }
        );
    }

    #[test]
    fn open_column_range_clamps_to_used_rows() {
        assert_eq!(
            spec("B:C").resolve(10, 42),
            Rect {
                start_col: 2,
                start_row: 1,
                end_col: 3,
                end_row: 42
            }
        );
    }

    #[test]
    fn open_row_range_clamps_to_used_columns() {
        assert_eq!(
            spec("2:4").resolve(6, 99),
            Rect {
                start_col: 1,
                start_row: 2,
                end_col: 6,
                end_row: 4
            }
        );
    }

    #[test]
    fn inverted_bounds_are_normalised() {
        assert_eq!(
            spec("D5:A1").resolve(99, 99),
            Rect {
                start_col: 1,
                start_row: 1,
                end_col: 4,
                end_row: 5
            }
        );
    }

    #[test]
    fn rejects_garbage_ranges() {
        // "1A:B2" holds a ':', so it must be read as a malformed range rather than
        // quietly accepted as a sheet name.
        for bad in ["A1:!!", "1A:B2", "A0", "ZZZZ1:B2"] {
            assert!(
                RangeSpec::parse(bad, OP).is_err(),
                "{bad:?} should not parse as a range"
            );
        }
    }

    #[test]
    fn existing_sheet_name_wins_over_cell_reading() {
        // "Sheet1" is both a valid cell ref (column SHEET, row 1) and the commonest tab
        // name there is. With the workbook to consult, the tab must win.
        let parsed = RangeSpec::parse_in("Sheet1", OP, &|name| name == "Sheet1").unwrap();
        assert_eq!(parsed.sheet.as_deref(), Some("Sheet1"));
        assert_eq!(parsed.start_col, None);

        // A short token that really is a cell ref stays one when no such tab exists.
        let parsed = RangeSpec::parse_in("B2", OP, &|_| false).unwrap();
        assert_eq!(parsed.sheet, None);
        assert_eq!(parsed.start_col, Some(2));
    }

    #[test]
    fn unknown_long_name_is_treated_as_a_sheet_not_a_column() {
        // Even absent from the workbook, `Sheet9` should surface as a missing tab rather
        // than a column-out-of-range complaint.
        let parsed = RangeSpec::parse_in("Sheet9", OP, &|_| false).unwrap();
        assert_eq!(parsed.sheet.as_deref(), Some("Sheet9"));
    }

    #[test]
    fn long_alphabetic_sheet_names_are_not_column_letters() {
        // Regression: a name like "Inventory" was parsed as column letters and overflowed.
        for name in ["Inventory", "Quarterly", "A", "AAAAAAAAA1"] {
            let parsed = spec(name);
            assert_eq!(parsed.sheet.as_deref(), Some(name));
            assert_eq!(parsed.start_col, None);
        }
    }

    #[test]
    fn last_valid_column_is_accepted() {
        assert_eq!(spec("XFD1").start_col, Some(MAX_COLUMN));
    }

    #[test]
    fn renders_a1_with_quoted_sheet() {
        let rect = Rect {
            start_col: 1,
            start_row: 1,
            end_col: 2,
            end_row: 3,
        };
        assert_eq!(rect.to_a1("Sheet1"), "'Sheet1'!A1:B3");
    }

    #[test]
    fn emitted_ranges_parse_back_to_the_same_sheet() {
        // Every response hands the caller a `range` string; it has to survive being fed
        // straight back in, including for tab names holding apostrophes or bangs.
        let rect = Rect {
            start_col: 1,
            start_row: 1,
            end_col: 2,
            end_row: 3,
        };
        for name in ["Sheet1", "Sh!eet'4", "Sheet''6", "Q1!Final", "it's here"] {
            let rendered = rect.to_a1(name);
            let reparsed = RangeSpec::parse(&rendered, OP)
                .unwrap_or_else(|err| panic!("{rendered:?} should reparse: {err}"));
            assert_eq!(reparsed.sheet.as_deref(), Some(name), "from {rendered:?}");
            assert_eq!(reparsed.resolve(1, 1), rect, "from {rendered:?}");
        }
    }

    #[test]
    fn sheet_names_default_to_one_tab() {
        assert_eq!(parse_sheet_names(None, OP).unwrap(), vec!["Sheet1"]);
    }

    #[test]
    fn sheet_names_are_split_and_trimmed() {
        assert_eq!(
            parse_sheet_names(Some("Summary, Q1 Data ,Notes"), OP).unwrap(),
            vec!["Summary", "Q1 Data", "Notes"]
        );
    }

    #[test]
    fn sheet_names_reject_duplicates_and_emptiness() {
        assert!(parse_sheet_names(Some("A,A"), OP).is_err());
        assert!(parse_sheet_names(Some(" , "), OP).is_err());
    }

    #[test]
    fn values_must_be_an_array_of_arrays() {
        assert!(parse_values(r#"[["a",1],["b",2]]"#, OP).is_ok());
        assert!(parse_values(r#"{"a":1}"#, OP).is_err());
        assert!(parse_values(r#"["a","b"]"#, OP).is_err());
        assert!(parse_values("not json", OP).is_err());
    }
}
