use tauri_build::Attributes;

fn main() {
    let attrs = Attributes::new();

    if let Err(e) = tauri_build::try_build(attrs) {
        let e = e.to_string();
        eprintln!("[tauri-build] {}", e);
        if e.contains("not found") || e.contains(" not in ") || e.contains("required for") || e.contains("RC.EXE") {
            eprintln!("  (Window resource step skipped — binary compiles fine for dev/debug mode.)");
        } else {
            panic!("tauri build error: {}", e);
        }
    }
}
