//! `__manifest__.py` parser — rustpython-parser 0.4.x AST.
//!
//! v0.4 AST key diffs:
//!   • Suite  = Vec<Stmt<R>>  (no `.body` wrapper)
//!   • Expr::Dict(ExprDict { keys: Vec<Option<Expr>>, values: Vec<Expr> })
//!   • Expr::Constant(ExprConstant { value: Constant, .. })     ← tuple variant
//!   • Constant::Str(String)
//!   • Identifier: Deref<Target = str>

use rustpython_parser::{ast, Parse};

/// Parsed result of an Odoo __manifest__.py.
#[derive(Debug, Clone, Default)]
pub struct ParsedManifest {
    pub module_name: String,
    pub depends: Vec<String>,
    pub external_deps_python: Vec<String>,
    pub external_deps_bin: Vec<String>,
}

/// Parse the string content of a `__manifest__.py` file.
pub fn parse_manifest(content: &str) -> Result<ParsedManifest, String> {
    let suite: ast::Suite = ast::Suite::parse(content, "<__manifest__.py>")
        .map_err(|e| format!("Python parse error: {e:?}"))?;

    let mut result = ParsedManifest::default();
    for stmt in &suite {
        if let ast::Stmt::Expr(e) = stmt {
            walk_expr(&e.value, &mut result);
        }
    }
    Ok(result)
}

// ═══════════════════════════════════════════════════════════════════════════════
// AST walk
// ═══════════════════════════════════════════════════════════════════════════════

fn walk_expr(expr: &ast::Expr, result: &mut ParsedManifest) {
    match expr {
        ast::Expr::Dict(d) => walk_dict_keys(d, result),
        ast::Expr::BoolOp(b) => {
            for v in &b.values { walk_expr(v, result); }
        }
        ast::Expr::BinOp(b) => {
            walk_expr(&b.left, result);
            walk_expr(&b.right, result);
        }
        ast::Expr::Call(c) => {
            if let ast::Expr::Name(name_expr) = c.func.as_ref() {
                if name_expr.id.as_str() == "dict" {
                    for kw in &c.keywords {
                        match kw.arg.as_deref() {
                            Some("depends") =>
                                python_list_to_strings(&kw.value, &mut result.depends),
                            Some("external_dependencies") =>
                                walk_ext_deps(&kw.value, result),
                            _ => {}
                        }
                    }
                }
            }
        }
        _ => {}
    }
}

fn walk_dict_keys(d: &ast::ExprDict, result: &mut ParsedManifest) {
    // d.keys = Vec<Option<Expr>>  (None = **kwargs unpacking)
    // d.values = Vec<Expr>
    for (key_opt, val) in d.keys.iter().zip(d.values.iter()) {
        let key: &str = match key_opt {
            Some(k) => const_str(k).unwrap_or(""),
            None => "",
        };
        match key {
            "depends" =>
                python_list_to_strings(val, &mut result.depends),
            "external_dependencies" =>
                walk_ext_deps(val, result),
            "name" => {
                if let Some(s) = const_str(val) { result.module_name = s.to_string(); }
            }
            _ => {}
        }
    }
}

fn walk_ext_deps(expr: &ast::Expr, result: &mut ParsedManifest) {
    match expr {
        ast::Expr::Dict(d) => {
            for (key_opt, val) in d.keys.iter().zip(d.values.iter()) {
                let key: &str = match key_opt {
                    Some(k) => const_str(k).unwrap_or(""),
                    None => "",
                };
                match key {
                    "python" =>
                        python_list_to_strings(val, &mut result.external_deps_python),
                    "bin" =>
                        python_list_to_strings(val, &mut result.external_deps_bin),
                    _ => {}
                }
            }
        }
        ast::Expr::Call(c) => {
            if let ast::Expr::Name(name_expr) = c.func.as_ref() {
                if name_expr.id.as_str() == "dict" {
                    for kw in &c.keywords {
                        match kw.arg.as_deref() {
                            Some("python") =>
                                python_list_to_strings(&kw.value, &mut result.external_deps_python),
                            Some("bin") =>
                                python_list_to_strings(&kw.value, &mut result.external_deps_bin),
                            _ => {}
                        }
                    }
                }
            }
        }
        _ => {}
    }
}

/// Collect `str` elements from a Python list or tuple expression into `out`.
fn python_list_to_strings(expr: &ast::Expr, out: &mut Vec<String>) {
    match expr {
        ast::Expr::List(lst) => {
            out.extend(lst.elts.iter().filter_map(const_str).map(|s| s.to_string()));
        }
        ast::Expr::Tuple(tup) => {
            out.extend(tup.elts.iter().filter_map(const_str).map(|s| s.to_string()));
        }
        _ => {}
    }
}

/// Extract a `&str` from `Expr::Constant(ExprConstant { value: Constant::Str(s) })`.
/// v0.4 uses the **tuple variant** `Expr::Constant(value_field)` not struct syntax.
fn const_str(expr: &ast::Expr) -> Option<&str> {
    if let ast::Expr::Constant(ExprConstant { value, .. }) = expr {
        if let ast::Constant::Str(s) = value { return Some(s.as_str()); }
    }
    None
}

/// ExprConstant is public in v0.4 but its fields are pub so this is fine.
#[allow(unused)]
type ExprConstant<'a> = ast::ExprConstant;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_depends() {
        let m = parse(r#"{
            'name': 'My Module',
            'depends': ['base', 'sale'],
        }"#);
        assert_eq!(m.depends, vec!["base", "sale"]);
        assert_eq!(m.module_name, "My Module");
    }

    #[test]
    fn ext_python() {
        let m = parse(r#"{
            'name': 'Barcode', 'depends': ['web'],
            'external_dependencies': {'python': ['qrcode', 'pyzbar']},
        }"#);
        assert!(m.external_deps_python.contains(&"qrcode".into()));
        assert!(m.external_deps_python.contains(&"pyzbar".into()));
    }

    #[test]
    fn ext_bin() {
        let m = parse(r#"{
            'name': 'PDF', 'depends': ['base'],
            'external_dependencies': {'bin': ['wkhtmltopdf']},
        }"#);
        assert_eq!(m.external_deps_bin, vec!["wkhtmltopdf"]);
    }

    #[test]
    fn no_depends() {
        let m = parse(r#"{'name': 'Y'}"#);
        assert!(m.depends.is_empty());
        assert_eq!(m.module_name, "Y");
    }

    #[track_caller]
    fn parse(s: &str) -> ParsedManifest {
        parse_manifest(s).expect("parse failed")
    }
}
