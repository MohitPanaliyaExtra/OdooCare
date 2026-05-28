use std::path::PathBuf;
use std::collections::{HashMap, HashSet};
use serde::Serialize;
use tauri::{Emitter, Manager};

mod zip_handler;
mod manifest_parser;
mod dependency_analyzer;
mod python_import_checker;
mod odoo17_ce;

use odoo17_ce::{ActivationStep, ExternalDep, MissingDep, UndeclaredImport, ODOO17_CE_PYTHON_PACKAGES};
use manifest_parser::ParsedManifest;
use dependency_analyzer::compute_activation_order;

// ── Types shared with the frontend via Tauri IPC ───────────────────────────────

#[derive(Serialize, Clone, Debug)]
pub struct AnalysisResult {
    pub zip_module_name: String,
    pub total_modules_found: usize,
    pub activation_order: Vec<ActivationStep>,
    pub external_dependencies: Vec<ExternalDep>,
    pub missing_dependencies: Vec<MissingDep>,
    pub undeclared_python_imports: Vec<UndeclaredImport>,
    pub warnings: Vec<String>,
}

// ── Progress events ────────────────────────────────────────────────────────────

#[derive(Clone, Serialize)]
struct ProgressEvent {
    stage: &'static str,
    message: String,
    percent: u8,
}

impl ProgressEvent {
    fn send<R: tauri::Runtime>(
        app: &tauri::AppHandle<R>,
        stage: &'static str,
        message: impl Into<String>,
        percent: u8,
    ) {
        let _ = app.emit("analyze-progress", Self {
            stage,
            message: message.into(),
            percent,
        });
    }
}

// ── Tauri command ──────────────────────────────────────────────────────────────

#[tauri::command]
async fn analyze_module_zip(
    app: tauri::AppHandle,
    zip_path: String,
) -> Result<AnalysisResult, String> {

    ProgressEvent::send(&app, "start", "Analyzing…", 0);

    let zip_pb = PathBuf::from(&zip_path);
    if !zip_pb.exists() {
        return Err(format!("File not found: {zip_path}"));
    }

    // 1. Extract zip + discover manifests
    ProgressEvent::send(&app, "extract", "Extracting zip…", 10);
    let (modules, _tmp_dir) = zip_handler::extract_and_discover(&zip_pb)
        .map_err(|e| format!("Zip error: {e}"))?;

    if modules.is_empty() {
        return Err("No __manifest__.py found in zip.".into());
    }
    ProgressEvent::send(
        &app, "discover",
        format!("Found {} module(s).", modules.len()), 30,
    );

    // 2. Parse all manifests
    let mut manifests: HashMap<String, ParsedManifest> = HashMap::new();
    for found in &modules {
        ProgressEvent::send(
            &app, "manifest",
            format!("Parsing {}", found.name), 35,
        );
        let content = std::fs::read_to_string(&found.manifest_path)
            .map_err(|e| format!("Read {}: {e}", found.manifest_path.display()))?;
        let m = manifest_parser::parse_manifest(&content)
            .map_err(|e| format!("{}: {e}", found.name))?;
        manifests.insert(found.name.clone(), m);
    }

    // 3. Dependency graph + topological sort
    let user_mod_names: HashSet<&str> =
        manifests.keys().map(|s| s.as_str()).collect();
    let mut user_module_deps: HashMap<&str, Vec<&str>> = HashMap::new();
    for (n, m) in &manifests {
        user_module_deps.insert(n.as_str(), m.depends.iter().map(|s| s.as_str()).collect());
    }

    let (activation_order, mut warnings) =
        compute_activation_order(&user_mod_names, &user_module_deps);

    ProgressEvent::send(
        &app, "sort",
        format!("{} activation steps.", activation_order.len()), 65,
    );

    // 4. External Python / binary deps from manifests
    let mut external_deps: Vec<ExternalDep> = Vec::new();
    for m in manifests.values() {
        for pkg in &m.external_deps_python {
            external_deps.push(ExternalDep {
                category: "python".into(),
                package_name: pkg.clone(),
                install_command: format!("pip install {pkg}"),
            });
        }
        for bin in &m.external_deps_bin {
            external_deps.push(ExternalDep {
                category: "binary".into(),
                package_name: bin.clone(),
                install_command: format!("Install: {bin}"),
            });
        }
    }

    // 5. Missing deps (activation_order entries with source = "missing")
    let missing_deps: Vec<MissingDep> = activation_order
        .iter()
        .filter(|s| s.source == "missing")
        .map(|s| MissingDep {
            module_name: s.module_name.clone(),
            source: "manifest-dep".into(),
        })
        .collect();

    // 6. Scan .py files for undeclared imports
    ProgressEvent::send(&app, "imports", "Scanning Python imports…", 80);

    let allowed_pkgs: HashSet<&str> =
        ODOO17_CE_PYTHON_PACKAGES.iter().map(|s| *s).collect();
    let mut undeclared: Vec<UndeclaredImport> = Vec::new();

    for found in &modules {
        let imps = python_import_checker::find_undeclared_python_imports(
            &found.dir_path, &allowed_pkgs,
        ).unwrap_or_default();
        for mut imp in imps {
            imp.module_name = found.name.clone();
            undeclared.push(imp);
        }
    }

    // 7. Warnings
    for m in &missing_deps {
        warnings.push(format!("MISSING: module '{}' not found anywhere.", m.module_name));
    }

    // 8. Assemble result
    let zip_name = zip_pb.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();

    ProgressEvent::send(&app, "done", "Done.", 100);

    Ok(AnalysisResult {
        zip_module_name: zip_name,
        total_modules_found: modules.len(),
        activation_order,
        external_dependencies: external_deps,
        missing_dependencies: missing_deps,
        undeclared_python_imports: undeclared,
        warnings,
    })
}

// ── Entry point ────────────────────────────────────────────────────────────────

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            analyze_module_zip,
        ])
        .setup(|app| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.emit("frontend-ready", "ok");
                let _ = window.show();
                let _ = window.set_focus();
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error running odoocare");
}
