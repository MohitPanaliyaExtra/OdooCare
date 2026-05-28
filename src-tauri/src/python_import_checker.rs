use std::path::Path;
use std::collections::HashSet;
use walkdir::WalkDir;
use pyimportparse::{parse_imports, Import};

use crate::odoo17_ce::UndeclaredImport;

/// Walk every `.py` file under `module_dir` and return any *undeclared*
/// third-party Python package not found in Odoo 17 CE `requirements.txt`
/// or in the Python stdlib.
pub fn find_undeclared_python_imports(
    module_dir: &Path,
    allowed: &HashSet<&str>,
) -> Result<Vec<UndeclaredImport>, std::io::Error> {
    use crate::odoo17_ce::PY_STDLIB;

    let mut found: Vec<UndeclaredImport> = Vec::new();

    for entry in WalkDir::new(module_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_type().is_file()
                && e.file_name().to_string_lossy().ends_with(".py")
        })
    {
        let file_path = entry.path().to_path_buf();
        let rel_path = file_path
            .strip_prefix(module_dir)
            .unwrap_or(&file_path)
            .to_string_lossy()
            .to_string();

        let src = std::fs::read_to_string(&file_path)?;
        let src = strip_leading_comments(&src);

        let imports: Vec<Import> = match parse_imports(&src) {
            Ok(v) => v,
            Err(_) => continue,
        };

        for imp in imports {
            let raw = imp.imported_object.trim();

            // Skip relative imports: ".foo" "..foo"
            let trimmed = raw.trim_start_matches('.');
            let top = first_segment(&trimmed);

            // Skip: empty | stdlib | allowed | odoo-internal | type-checking-only
            if top.is_empty()
                || PY_STDLIB.contains(&top)
                || allowed.contains(top)
                || looks_odoo_internal(trimmed)
                || imp.typechecking_only
            {
                continue;
            }

            found.push(UndeclaredImport {
                module_name:  String::new(), // set by caller
                file_path:    rel_path.clone(),
                package_name: top.into(),
            });
        }
    }

    // In-place dedup by (package_name, file_path)
    let mut deduped: Vec<UndeclaredImport> = Vec::new();
    let mut seen: HashSet<(String, String)> = HashSet::new();
    for item in found {
        let key = (item.package_name.clone(), item.file_path.clone());
        if seen.insert(key) {
            deduped.push(item);
        }
    }

    Ok(deduped)
}

/// First dotted segment: "x.y.z" → "x"
fn first_segment(s: &str) -> &str {
    s.split('.').next().unwrap_or("")
}

/// Strip consecutive comment lines from the top of a file.
fn strip_leading_comments(s: &str) -> &str {
    let mut start = 0usize;
    for line in s.lines() {
        let t = line.trim_start();
        if t.starts_with('#') { start += line.len() + 1; } else { break; }
    }
    &s[start..]
}

/// Returns true for imports that look like Odoo internal modules.
fn looks_odoo_internal(name: &str) -> bool {
    let lc = name.to_lowercase();
    lc.starts_with("odoo") || lc.starts_with("openerp")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn first_segment_works() {
        assert_eq!(first_segment("requests"), "requests");
        assert_eq!(first_segment("x.y.z"), "x");
        assert_eq!(first_segment(""), "");
    }

    #[test]
    fn strip_comments_works() {
        let src = "# header\nimport os\n";
        let stripped = strip_leading_comments(src);
        assert!(stripped.contains("import os"));
    }

    #[test]
    fn odoo_internal_detected() {
        assert!(looks_odoo_internal("odoo.addons"));
        assert!(!looks_odoo_internal("requests"));
    }
}
