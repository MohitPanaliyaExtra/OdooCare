<script lang="ts">
    import { onMount } from 'svelte'
    import { invoke } from '@tauri-apps/api/core'
    import { open } from '@tauri-apps/plugin-dialog'
    import { listen } from '@tauri-apps/api/event'

    // ── State ────────────────────────────────────────────────────────────────
    let analyzing   = $state(false)
    let progress    = $state(0)
    let progressMsg = $state('')

    // Result state
    let result    = $state<AnalysisResult | null>(null)
    let errorMsg  = $state('')

    let dragOver   = $state(false)
    let viewMode   = $state<'form' | 'results'>('form')

    // Filter selected panel tabs
    let activeTab    = $state<'all' | 'internal' | 'ce' | 'missing'>('all')

    interface ActivationStep {
        order: number
        module_name: string
        source: string  // 'internal' | 'external-ce' | 'missing'
    }

    interface ExternalDep {
        category: string  // 'python' | 'binary'
        package_name: string
        install_command: string
    }

    interface UndeclaredImport {
        module_name: string
        file_path: string
        package_name: string
    }

    // helper type matches backend AnalysisResult
    interface AnalysisResult {
        zip_module_name: string
        total_modules_found: number
        activation_order: ActivationStep[]
        external_dependencies: ExternalDep[]
        missing_dependencies: { module_name: string; source: string }[]
        undeclared_python_imports: UndeclaredImport[]
        warnings: string[]
    }

    // ── Lifecycle ─────────────────────────────────────────────────────────────
    onMount(() => {
        const unlistenFrontend = listen('frontend-ready', () => {
            console.log('Backend ready')
        })

        const unlistenProgress = listen('analyze-progress', (event) => {
            const { message, percent } = event.payload
            progressMsg = message
            progress    = percent
        })

        return () => {
            unlistenFrontend()
            unlistenProgress()
        }
    })

    // ── File selection ────────────────────────────────────────────────────────
    let selectedFile: File | null = $state(null)

    async function handleBrowse() {
        const selected = await open({
            multiple: false,
            filters: [{ name: 'Zip', extensions: ['zip'] }],
        })
        if (selected && typeof selected === 'string') {
            await runAnalysis(selected)
        }
    }

    async function handleDrop(e: DragEvent) {
        e.preventDefault()
        dragOver = false
        const file = e.dataTransfer?.files?.[0]
        if (file && file.name.endsWith('.zip')) {
            selectedFile = file
            await runAnalysis(file)
        }
    }

    async function runAnalysis(pathOrFile: string | File) {
        analyzing   = true
        errorMsg    = ''
        progress    = 0
        progressMsg = 'Starting…'
        result      = null
        viewMode    = 'form'

        try {
            const zipPath = typeof pathOrFile === 'string'
                ? pathOrFile
                : '' // File objects via drag-drop need the Tauri file picker path

            const res = await invoke<AnalysisResult>('analyze_module_zip', {
                zipPath: typeof pathOrFile === 'string' ? pathOrFile : await fileToPath(pathOrFile),
            })
            result   = res
            viewMode = 'results'
        } catch (err: any) {
            errorMsg = err || 'An unknown error occurred.'
        } finally {
            analyzing = false
        }
    }

    async function fileToPath(file: File): Promise<string> {
        // Extract a readable path via Tauri dialog – but drag-and-drop gives us
        // a File object with a path already (webkit relativePath blob in Chrome).
        // We fall back to the Tauri file path convention marker.
        return (file as any).path || ''
    }

    // ── Results helpers ───────────────────────────────────────────────────────
    let filteredSteps = $derived(
        result?.activation_order.filter((s) =>
            activeTab === 'all'     ? true :
            activeTab === 'internal'   ? s.source === 'internal' :
            activeTab === 'ce'         ? s.source === 'external-ce' :
            s.source === 'missing'
        ) ?? []
    )

    function downloadReport() {
        if (!result) return
        const md = exportMarkdown(result)
        const blob = new Blob([md], { type: 'text/markdown' })
        const a = document.createElement('a')
        a.href = URL.createObjectURL(blob)
        a.download = `${result.zip_module_name}_odoo-deps.md`
        a.click()
    }

    function copyReport() {
        if (!result) return
        navigator.clipboard.writeText(exportMarkdown(result))
    }

    function exportMarkdown(res: AnalysisResult): string {
        let md = `# OdooCARE — Dependency Report: ${res.zip_module_name}\n\n`
        md += `## Module Activation Order\n\n`
        for (const s of res.activation_order) {
            const label = s.source === 'internal' ? '📦 SHIPPED'
                        : s.source === 'external-ce' ? '🔵 ODOO CE'
                        : '❌ MISSING'
            md += `${s.order}. **${s.module_name}** \`[${label}]\`\n`
        }
        if (res.external_dependencies.length) {
            md += `\n## External Dependencies\n\n`
            for (const d of res.external_dependencies) {
                md += `- **[${d.category.toUpperCase()}]** \`${d.package_name}\` — ${d.install_command}\n`
            }
        }
        if (res.undeclared_python_imports.length) {
            md += `\n## Undeclared Python Imports\n\n`
            for (const im of res.undeclared_python_imports) {
                md += `- In **\`${im.module_name}/${im.file_path}\`**: \`${im.package_name}\`\n`
            }
        }
        if (res.warnings.length) {
            md += `\n## Warnings\n\n`
            for (const w of res.warnings) md += `- ⚠️ ${w}\n`
        }
        return md
    }

    function reset() {
        result   = null
        errorMsg = ''
        viewMode = 'form'
        activeTab = 'all'
    }

    function activateTab(t: 'all' | 'internal' | 'ce' | 'missing') {
        activeTab = t
    }
</script>

<!-- ═══════════════════════════════════════════════════════════════════════════ -->
<!--  HEADER                                                                   -->
<!-- ═══════════════════════════════════════════════════════════════════════════ -->
<header class="app-header">
    <span class="app-logo">⚙ OdooCARE</span>
    <span class="app-sub">Odoo 17 Community — Module Dependency Analyzer</span>
</header>

<!-- ═══════════════════════════════════════════════════════════════════════════ -->
<!--  BODY                                                                     -->
<!-- ═══════════════════════════════════════════════════════════════════════════ -->
<div class="app-body">

    <!-- ─── FORM / DROP ZONE ─────────────────────────────────────────────── -->
    {#if viewMode === 'form'}
        <div
            class="drop-zone"
            class:drag-over={dragOver}
            role="button"
            tabindex="0"
            ondragover={(e) => { e.preventDefault(); dragOver = true }}
            ondragleave={() => { dragOver = false }}
            ondrop={handleDrop}
            onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); if (!analyzing) handleBrowse(); } }}
            onclick={() => !dragOver && !analyzing && handleBrowse()}
        >
            <div class="drop-icon">
                {#if analyzing}
                    ⏳
                {:else}
                    📦
                {/if}
            </div>

            {#if analyzing}
                <div style="width: 240px; margin-top: 4px;">
                    <div class="progress-bar-wrap">
                        <div class="progress-bar-fill" style="width: {progress}%"></div>
                    </div>
                    <p style="color: var(--text-400); font-size: 13px; margin-top: 8px; text-align: center;">
                        {progressMsg}
                    </p>
                </div>
            {:else}
                <p class="drop-title">
                    Drop your Odoo module ZIP here
                </p>
                <p class="drop-sub">
                    Upload a <code style="background: var(--bg-600); padding: 2px 6px; border-radius: 4px;">.zip</code>
                    containing one or more Odoo modules.<br />
                    We will parse all <code style="background: var(--bg-600); padding: 2px 6px; border-radius: 4px;">__manifest__.py</code>
                    files, resolve the dependency graph, and detect undeclared Python imports.
                </p>
                <button class="drop-btn" onclick={(e) => { e.stopPropagation(); handleBrowse() }}>
                    Browse ZIP File…
                </button>
            {/if}
        </div>

        {#if errorMsg}
            <div style="padding: 14px 20px; margin: 0 20px; background: rgba(239,68,68,.1); border: 1px solid rgba(239,68,68,.3); border-radius: var(--radius); color: var(--red); font-size: 13px;">
                ⚠️ {errorMsg}
            </div>
        {/if}

    <!-- ─── RESULTS ─────────────────────────────────────────────────────── -->
    {:else if result}
        {#if result.warnings.length > 0}
            <div class="warn-bar">
                <div class="sec-title">⚠ Warnings ({result.warnings.length})</div>
                {#each result.warnings as w}
                    <div class="warn-chip">{w}</div>
                {/each}
            </div>
        {/if}

        <div class="results-layout">

            <!-- ── Tabs ──────────────────────────────────────────────────── -->
            <div style="display: flex; gap: 4px; flex-shrink: 0;">
                <button
                    class="btn-outline"
                    style="padding: 4px 14px; font-size: 12px; border-radius: 6px;"
                    class:btn-primary={activeTab === 'all'}
                    onclick={() => activateTab('all')}
                >
                    All ({result.activation_order.length})
                </button>
                <button
                    class="btn-outline"
                    style="padding: 4px 14px; font-size: 12px; border-radius: 6px;"
                    class:btn-primary={activeTab === 'internal'}
                    onclick={() => activateTab('internal')}
                >
                    📦 Internal ({result.activation_order.filter(s => s.source === 'internal').length})
                </button>
                <button
                    class="btn-outline"
                    style="padding: 4px 14px; font-size: 12px; border-radius: 6px;"
                    class:btn-primary={activeTab === 'ce'}
                    onclick={() => activateTab('ce')}
                >
                    🔵 Odoo CE ({result.activation_order.filter(s => s.source === 'external-ce').length})
                </button>
                <button
                    class="btn-outline"
                    style="padding: 4px 14px; font-size: 12px; border-radius: 6px;"
                    class:btn-primary={activeTab === 'missing'}
                    onclick={() => activateTab('missing')}
                >
                    ❌ Missing ({result.activation_order.filter(s => s.source === 'missing').length})
                </button>
            </div>

            <!-- ── Three-panel grid ────────────────────────────────────────── -->
            <div class="panels">

                <!-- Panel 1 - Activation order -->
                <div class="step-panel">
                    <div class="panel-label">
                        📦 Activation Order ({filteredSteps.length} of {result.activation_order.length})
                    </div>

                    {#each filteredSteps as step}
                        <div class="step-item">
                            <div class="step-num">{step.order}</div>
                            <span class="step-name">{step.module_name}</span>
                            <span
                                class="badge"
                                class:badge-internal={step.source === 'internal'}
                                class:badge-external={step.source === 'external-ce'}
                                class:badge-missing={step.source === 'missing'}
                            >
                                {step.source === 'internal' ? 'In ZIP' :
                                 step.source === 'external-ce' ? 'Odoo CE' :
                                 'MISSING'}
                            </span>
                        </div>
                    {/each}
                </div>

                <!-- Panel 2 - External deps -->
                <div class="ext-panel">
                    <div class="panel-label">
                        📋 Deps ({result.external_dependencies.length})
                    </div>

                    {#if result.external_dependencies.length === 0}
                        <p style="color: var(--text-400); font-size: 13px;">No external deps found in manifests.</p>
                    {:else}
                        <table class="dep-table">
                            <thead>
                                <tr>
                                    <th>#</th><th>Type</th><th>Package</th><th>Install</th>
                                </tr>
                            </thead>
                            <tbody>
                                {#each result.external_dependencies as dep, i}
                                    <tr>
                                        <td style="color: var(--text-400); font-size: 11px">{i + 1}</td>
                                        <td>
                                            <span class={dep.category === 'python' ? 'cat-py' : 'cat-bin'}>
                                                {dep.category.toUpperCase()}
                                            </span>
                                        </td>
                                        <td style="font-weight: 600">{dep.package_name}</td>
                                        <td><code class="pip-cmd">{dep.install_command}</code></td>
                                    </tr>
                                {/each}
                            </tbody>
                        </table>
                    {/if}
                </div>

                <!-- Panel 3 - Undeclared imports + missing -->
                <div class="miss-panel">
                    <div class="panel-label">
                        🔎 Undeclared Python Imports ({result.undeclared_python_imports.length})
                    </div>

                    {#each result.undeclared_python_imports as imp}
                        <div class="undeclared-item">
                            <div class="pkg-name">{imp.package_name}</div>
                            <div class="file-path">
                                {imp.module_name}/{(imp.file_path)}
                            </div>
                        </div>
                    {/each}
                    {#if result.undeclared_python_imports.length === 0}
                        <p style="color: var(--text-400); font-size: 13px; margin-top: 4px;">
                            ✅ All imports satisfied by Odoo CE requirements.txt.
                        </p>
                    {/if}

                    {#if result.missing_dependencies.length > 0}
                        <div class="panel-label" style="margin-top: 14px;">
                            ❌ Missing Odoo Modules ({result.missing_dependencies.length})
                        </div>
                        {#each result.missing_dependencies as m}
                            <div class="miss-item">
                                <span>❌</span>
                                <div>
                                    <div style="font-weight: 700; color: var(--red);">{m.module_name}</div>
                                    <div style="font-size: 11px; color: var(--text-400);">
                                        Export from {m.source}
                                    </div>
                                </div>
                            </div>
                        {/each}
                    {/if}
                </div>

            </div><!-- .panels -->

        </div><!-- .results-layout -->

        <!-- ─── Action bar ────────────────────────────────────────────────── -->
        <div class="done-bar">
            <button class="btn-outline" onclick={copyReport}>
                📋 Copy Report to Clipboard
            </button>
            <button class="btn-outline" onclick={downloadReport}>
                ↓ Download Markdown Report
            </button>
            <button class="btn-outline" onclick={reset}>
                ← Analyse Another ZIP
            </button>
            <span style="margin-left: auto; color: var(--text-400); font-size: 12px;">
                {result.activation_order.length} activation steps · 
                {result.external_dependencies.length} external deps ·
                {result.undeclared_python_imports.length} undeclared imports
            </span>
        </div>
    {/if}

</div><!-- .app-body -->
