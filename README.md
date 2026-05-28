# OdooCARE

**Odoo Community App Reliability Evaluator** — A desktop app that analyzes Odoo module ZIPs before installation to prevent crashes.

## Problem

Odoo Community Edition lacks an app store like Odoo Enterprise. Users install modules from OCA, third-party vendors, or custom builds — often with incomplete or missing dependencies. Installing a bad module can crash the entire Odoo instance.

## Solution

OdooCARE is a **Rust + Tauri + Svelte** desktop app that analyzes a module ZIP file and tells you:

- **Activation order** — Topological sort of all modules in the ZIP and their Odoo CE dependencies
- **Missing dependencies** — Modules declared in `depends` that aren't in the ZIP or Odoo CE
- **External dependencies** — Python packages and binaries required by the module
- **Undeclared Python imports** — Imports found in `.py` files that aren't in the module's manifest or Odoo CE's requirements

No more blind installations. Test first, then deploy.

## Features

- **Drag-and-drop or browse** for Odoo module ZIP files
- Parses all `__manifest__.py` files inside the archive
- Resolves dependency graph against **Odoo 17 CE** official module list (300+ modules)
- Scans Python source for undeclared imports
- Exports detailed **Markdown report**
- Desktop-native (Windows, macOS, Linux)

## Tech Stack

- **Backend:** Rust + Tauri 2
- **Frontend:** Svelte 5 + Tailwind CSS 4 + Vite
- **Dependency resolution:** petgraph (topological sort)
- **Python parsing:** rustpython-parser

## Build

```bash
# Prerequisites: Rust, Node.js, and Tauri CLI
cd src && npm install
cd ../src-tauri && npm install
cargo tauri build
```

## License

MIT
