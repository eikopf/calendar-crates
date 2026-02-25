use std::path::PathBuf;

use calico::parser::{component::calendar, escaped::AsEscaped};
use winnow::Parser;
use winnow::combinator::repeat;
use winnow::token::one_of;

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn collect_ics_files(dir: &std::path::Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if !dir.exists() {
        return files;
    }
    for entry in walkdir(dir) {
        if entry.extension().is_some_and(|e| e == "ics") {
            files.push(entry);
        }
    }
    files.sort();
    files
}

fn walkdir(dir: &std::path::Path) -> Vec<PathBuf> {
    let mut result = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                result.extend(walkdir(&path));
            } else {
                result.push(path);
            }
        }
    }
    result
}

#[test]
#[ignore]
fn parse_corpus() {
    let fixtures = fixtures_dir();
    let files = collect_ics_files(&fixtures);

    if files.is_empty() {
        eprintln!("No .ics fixtures found. Run `just fetch-fixtures` to download them.");
        return;
    }

    let mut total = 0;
    let mut passed = 0;
    let mut partial = 0;
    let mut failed = 0;

    let mut failures: Vec<(String, String)> = Vec::new();
    let mut partials: Vec<(String, usize, usize)> = Vec::new();

    for file in &files {
        total += 1;
        let rel = file.strip_prefix(&fixtures).unwrap_or(file);
        let name = rel.display().to_string();

        let input = match std::fs::read_to_string(file) {
            Ok(s) => s,
            Err(e) => {
                failed += 1;
                failures.push((name, format!("read error: {e}")));
                continue;
            }
        };

        let mut escaped = input.as_escaped();
        let mut calendars_parsed = 0u32;
        let mut file_failed = false;
        loop {
            // Skip blank lines between calendar objects
            let _ = repeat::<_, _, (), _, _>(0.., one_of::<_, _, ()>(('\r', '\n')))
                .parse_next(&mut escaped);

            if escaped.is_empty() {
                break;
            }

            match calendar::<_, winnow::error::InputError<_>>.parse_next(&mut escaped) {
                Ok(_cal) => {
                    calendars_parsed += 1;
                }
                Err(e) => {
                    if calendars_parsed > 0 {
                        partial += 1;
                        partials.push((name.clone(), input.len(), escaped.len()));
                    } else {
                        failed += 1;
                        failures.push((name.clone(), format!("{e:?}")));
                    }
                    file_failed = true;
                    break;
                }
            }
        }
        if !file_failed {
            passed += 1;
        }
    }

    // Print report
    eprintln!("\n=== Corpus Test Results ===");
    eprintln!("Total: {total}  Passed: {passed}  Partial: {partial}  Failed: {failed}");
    eprintln!();

    if !partials.is_empty() {
        eprintln!("--- Partial parses (input not fully consumed) ---");
        for (name, input_len, rem_len) in &partials {
            let consumed_pct = 100.0 * (1.0 - (*rem_len as f64 / *input_len as f64));
            eprintln!("  {name}: {consumed_pct:.1}% consumed ({rem_len} bytes remaining)");
        }
        eprintln!();
    }

    if !failures.is_empty() {
        eprintln!("--- Failures ---");
        for (name, err) in &failures {
            eprintln!("  {name}: {err}");
        }
        eprintln!();
    }

    // Don't assert — this is a diagnostic test. Just report.
    // As the parser improves, failures should decrease.
    eprintln!("Pass rate: {:.1}%", 100.0 * (passed as f64 / total as f64));
}
