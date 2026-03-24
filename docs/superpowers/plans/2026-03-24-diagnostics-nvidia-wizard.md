# System Diagnostics, NVIDIA Smart Defaults & Setup Wizard

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Three-phase plan: (1) Developer diagnostics tool showing saved vs runtime settings, (2) NVIDIA-only Wayland smart defaults, (3) First-run setup wizard.

**Architecture:** Phase 1 adds a new subsection to Developer Mode in SettingsView with a tabular comparison of saved vs runtime state, plus export. Phase 2 adds GPU detection logic to `apply_linux_webkit_workarounds()`. Phase 3 adds a multi-step wizard modal shown on first launch.

**Tech Stack:** SvelteKit (Svelte 5 runes), Rust/Tauri IPC, SQLite settings DBs, i18n (5 locales)

---

## Phase 1: Developer Diagnostics Tool

### Task 1.1: Backend — New `v2_get_runtime_diagnostics` command

**Files:**
- Modify: `src-tauri/src/commands_v2.rs`
- Modify: `src-tauri/src/lib.rs` (register command)

This single command returns ALL diagnostic data in one IPC call. No new crates needed — it reads from existing state/stores.

- [ ] **Step 1: Define the response struct in commands_v2.rs**

```rust
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeDiagnostics {
    // Audio: saved settings
    pub audio_output_device: Option<String>,
    pub audio_backend_type: Option<String>,
    pub audio_exclusive_mode: bool,
    pub audio_dac_passthrough: bool,
    pub audio_preferred_sample_rate: Option<u32>,
    pub audio_alsa_plugin: Option<String>,
    pub audio_alsa_hardware_volume: bool,
    pub audio_normalization_enabled: bool,
    pub audio_normalization_target_lufs: f32,
    pub audio_gapless_enabled: bool,
    pub audio_pw_force_bitperfect: bool,
    pub audio_stream_buffer_seconds: u8,
    pub audio_streaming_only: bool,

    // Audio: runtime
    pub runtime_hardware_sample_rate: Option<u32>,
    pub runtime_hardware_format: Option<String>,
    pub runtime_playback_active: bool,

    // Graphics: saved settings
    pub gfx_hardware_acceleration: bool,
    pub gfx_force_x11: bool,
    pub gfx_gdk_scale: Option<String>,
    pub gfx_gdk_dpi_scale: Option<String>,
    pub gfx_gsk_renderer: Option<String>,

    // Graphics: runtime (what actually applied at startup)
    pub runtime_using_fallback: bool,
    pub runtime_is_wayland: bool,
    pub runtime_has_nvidia: bool,
    pub runtime_has_amd: bool,
    pub runtime_has_intel: bool,
    pub runtime_is_vm: bool,
    pub runtime_hw_accel_enabled: bool,
    pub runtime_force_x11_active: bool,

    // Developer settings
    pub dev_force_dmabuf: bool,

    // Environment variables (what WebKit actually sees)
    pub env_webkit_disable_dmabuf: Option<String>,
    pub env_webkit_disable_compositing: Option<String>,
    pub env_gdk_backend: Option<String>,
    pub env_gsk_renderer: Option<String>,
    pub env_libgl_always_software: Option<String>,
    pub env_wayland_display: Option<String>,
    pub env_xdg_session_type: Option<String>,

    // App info
    pub app_version: String,
    pub webkit_gtk_version: String,  // if detectable
}
```

- [ ] **Step 2: Implement the command**

```rust
#[tauri::command]
pub fn v2_get_runtime_diagnostics(
    audio_state: State<'_, AudioSettingsState>,
    graphics_state: State<'_, GraphicsSettingsState>,
    developer_state: State<'_, DeveloperSettingsState>,
    app_state: State<'_, AppState>,
) -> Result<RuntimeDiagnostics, RuntimeError> {
    // Read audio settings from state
    let audio = crate::config::audio_settings::get_audio_settings(audio_state.inner())
        .unwrap_or_default();

    // Read graphics settings from state
    let gfx = crate::config::graphics_settings::get_graphics_settings(graphics_state.inner())
        .unwrap_or_default();

    // Read graphics startup status (static atomics)
    let gfx_status = crate::config::graphics_settings::get_graphics_startup_status();

    // Read developer settings
    let dev = crate::config::developer_settings::get_developer_settings(developer_state.inner())
        .unwrap_or_default();

    // Read environment variables
    let env_var = |name: &str| std::env::var(name).ok();

    Ok(RuntimeDiagnostics {
        audio_output_device: audio.output_device.clone(),
        audio_backend_type: audio.backend_type.map(|b| format!("{:?}", b)),
        audio_exclusive_mode: audio.exclusive_mode,
        audio_dac_passthrough: audio.dac_passthrough,
        audio_preferred_sample_rate: audio.preferred_sample_rate,
        audio_alsa_plugin: audio.alsa_plugin.map(|p| format!("{:?}", p)),
        audio_alsa_hardware_volume: audio.alsa_hardware_volume,
        audio_normalization_enabled: audio.normalization_enabled,
        audio_normalization_target_lufs: audio.normalization_target_lufs,
        audio_gapless_enabled: audio.gapless_enabled,
        audio_pw_force_bitperfect: audio.pw_force_bitperfect,
        audio_stream_buffer_seconds: audio.stream_buffer_seconds,
        audio_streaming_only: audio.streaming_only,

        runtime_hardware_sample_rate: None, // filled by frontend if playing
        runtime_hardware_format: None,
        runtime_playback_active: false,

        gfx_hardware_acceleration: gfx.hardware_acceleration,
        gfx_force_x11: gfx.force_x11,
        gfx_gdk_scale: gfx.gdk_scale,
        gfx_gdk_dpi_scale: gfx.gdk_dpi_scale,
        gfx_gsk_renderer: gfx.gsk_renderer,

        runtime_using_fallback: gfx_status.using_fallback,
        runtime_is_wayland: gfx_status.is_wayland,
        runtime_has_nvidia: gfx_status.has_nvidia,
        runtime_has_amd: gfx_status.has_amd,
        runtime_has_intel: gfx_status.has_intel,
        runtime_is_vm: gfx_status.is_vm,
        runtime_hw_accel_enabled: gfx_status.hardware_accel_enabled,
        runtime_force_x11_active: gfx_status.force_x11_active,

        dev_force_dmabuf: dev.force_dmabuf,

        env_webkit_disable_dmabuf: env_var("WEBKIT_DISABLE_DMABUF_RENDERER"),
        env_webkit_disable_compositing: env_var("WEBKIT_DISABLE_COMPOSITING_MODE"),
        env_gdk_backend: env_var("GDK_BACKEND"),
        env_gsk_renderer: env_var("GSK_RENDERER"),
        env_libgl_always_software: env_var("LIBGL_ALWAYS_SOFTWARE"),
        env_wayland_display: env_var("WAYLAND_DISPLAY"),
        env_xdg_session_type: env_var("XDG_SESSION_TYPE"),

        app_version: env!("CARGO_PKG_VERSION").to_string(),
        webkit_gtk_version: "unknown".to_string(), // WebKitGTK doesn't expose this easily
    })
}
```

- [ ] **Step 3: Register in lib.rs**

Add `commands_v2::v2_get_runtime_diagnostics` to the `.invoke_handler()` chain.

- [ ] **Step 4: Verify compilation**

Run: `cd src-tauri && cargo check`
Expected: compiles clean

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/commands_v2.rs src-tauri/src/lib.rs
git commit -m "feat: add v2_get_runtime_diagnostics command

Single IPC call returns all saved settings, runtime state,
and active environment variables for developer diagnostics.

cc"
```

---

### Task 1.2: Frontend — Diagnostics panel in Developer Mode

**Files:**
- Create: `src/lib/components/DiagnosticsPanel.svelte`
- Modify: `src/lib/components/views/SettingsView.svelte` (import + render)
- Modify: `src/lib/i18n/locales/en.json`
- Modify: `src/lib/i18n/locales/es.json`
- Modify: `src/lib/i18n/locales/de.json`
- Modify: `src/lib/i18n/locales/fr.json`
- Modify: `src/lib/i18n/locales/pt.json`

- [ ] **Step 1: Create DiagnosticsPanel.svelte**

A self-contained component that:
- Calls `v2_get_runtime_diagnostics` on mount
- Optionally calls `v2_get_hardware_audio_status` for live playback info
- Renders 3 collapsible tables: Audio, Graphics, Environment
- Each row: Setting Name | Saved Value | Runtime Value | Match indicator
- Has a "Refresh" button and "Export" button (copy JSON to clipboard)

Key patterns:
- Use `$state()` for diagnostics data
- Use `invoke()` from `@tauri-apps/api/core`
- Use `writeText` from `@tauri-apps/plugin-clipboard-manager` for export
- Use `$t()` for labels (but setting names can stay English — they're technical identifiers)
- NO `$t()` inside `$derived()` (ADR-001)
- NO `t` as variable name

The component should be ~200-300 lines. Three table sections:

**Audio section rows:**
| Setting | Saved | Runtime | Status |
|---------|-------|---------|--------|
| Output Device | "DacMagic Plus" | — | — |
| Backend | "PipeWire" | — | — |
| Exclusive Mode | true | — | — |
| DAC Passthrough | true | — | — |
| Sample Rate | 192000 | 44100 | mismatch (if playing) |
| ... | ... | ... | ... |

**Graphics section rows:**
| Setting | Saved | Runtime | Status |
|---------|-------|---------|--------|
| Hardware Acceleration | ON | ON | match |
| Force DMA-BUF | ON | WEBKIT_DISABLE_DMABUF=unset | match |
| Force X11 | ON | GDK_BACKEND=x11 | match |
| GSK Renderer | vulkan | GSK_RENDERER=vulkan | match |
| GPU: NVIDIA | — | detected | info |
| GPU: Intel | — | detected | info |
| Wayland | — | yes | info |

**Environment section rows:**
Raw display of all env vars read.

- [ ] **Step 2: Add i18n keys to all 5 locale files**

Keys to add under `settings.developer.diagnostics`:
```json
{
  "title": "System Diagnostics",
  "description": "Compare saved settings against what is actually running",
  "refresh": "Refresh",
  "export": "Export to Clipboard",
  "exported": "Copied!",
  "sectionAudio": "Audio",
  "sectionGraphics": "Graphics & Rendering",
  "sectionEnv": "Environment Variables",
  "colSetting": "Setting",
  "colSaved": "Saved",
  "colRuntime": "Runtime",
  "colStatus": "Status",
  "statusMatch": "OK",
  "statusMismatch": "Mismatch",
  "statusInfo": "Info",
  "notPlaying": "Not playing",
  "notSet": "Not set"
}
```

Add equivalent translations to es.json, de.json, fr.json, pt.json.

- [ ] **Step 3: Import and render in SettingsView.svelte**

In the Developer Mode section (after the Qobuz Connect Dev Tools subsection, around line 5656), add:

```svelte
<DiagnosticsPanel />
```

Import at top of `<script>`:
```typescript
import DiagnosticsPanel from '../DiagnosticsPanel.svelte';
```

- [ ] **Step 4: Verify with svelte-check**

Run: `npx svelte-check --threshold error`
Expected: 0 errors

- [ ] **Step 5: Commit**

```bash
git add src/lib/components/DiagnosticsPanel.svelte \
  src/lib/components/views/SettingsView.svelte \
  src/lib/i18n/locales/*.json
git commit -m "feat: add system diagnostics panel to Developer Mode

Tabular view comparing saved vs runtime settings for audio,
graphics, and environment variables. Includes export to clipboard.

cc"
```

---

## Phase 2: NVIDIA Smart Defaults

### Task 2.1: GPU-aware startup defaults

**Files:**
- Modify: `src-tauri/src/lib.rs` (`apply_linux_webkit_workarounds`)
- Modify: `src-tauri/src/main.rs` (ensure GPU detection runs before workarounds)

**Context files to read first:**
- `src-tauri/src/main.rs:41-81` (GPU detection functions)
- `src-tauri/src/config/graphics_settings.rs` (settings + atomics)

**Critical rule:** Users who already have a stable setup MUST NOT be affected. The logic only changes defaults for users who have NOT explicitly configured their graphics settings (i.e., all values are at DB defaults).

- [ ] **Step 1: Add `is_default_config()` to GraphicsSettingsStore**

```rust
/// Returns true if all settings are at their DB defaults (user never configured).
pub fn is_default_config(&self) -> Result<bool, String> {
    let settings = self.get_settings()?;
    Ok(
        !settings.hardware_acceleration  // DB default is 0 (false)
        && !settings.force_x11
        && settings.gdk_scale.is_none()
        && settings.gdk_dpi_scale.is_none()
        && settings.gsk_renderer.is_none()
    )
}
```

Add to `GraphicsSettingsStore` impl block, and also add a readonly version:
```rust
// In new_readonly() returned store, get_settings() already works
```

- [ ] **Step 2: Modify apply_linux_webkit_workarounds to detect GPU and apply smart defaults**

After reading saved settings but before applying env vars, add:

```rust
// Smart defaults for unconfigured users based on GPU detection
if is_default_config {
    let has_nvidia = detect_nvidia();  // reuse from main.rs
    let has_intel = detect_intel();
    let has_amd = detect_amd();
    let is_wayland = std::env::var("WAYLAND_DISPLAY").is_ok()
        || std::env::var("XDG_SESSION_TYPE").map(|v| v == "wayland").unwrap_or(false);

    if has_nvidia && !has_intel && !has_amd && is_wayland {
        // NVIDIA-only on Wayland: enable compositing (software rendering is terrible)
        log::info!("NVIDIA-only Wayland detected (unconfigured): enabling compositing mode");
        // Don't set WEBKIT_DISABLE_COMPOSITING_MODE
        // Still disable DMA-BUF (crash-prone on NVIDIA Wayland)
        hw_accel = true;
    }
}
```

Decision matrix for **unconfigured** users only:

| GPU | Session | Compositing | DMA-BUF | Rationale |
|-----|---------|-------------|---------|-----------|
| NVIDIA-only + Wayland | wayland | ON | OFF | No iGPU fallback, needs compositing |
| NVIDIA + Intel/AMD | any | OFF | OFF | iGPU handles WebKit fine |
| Intel/AMD only | any | OFF | OFF | Safe default |
| VM | any | OFF | OFF | No real GPU |

- [ ] **Step 3: Extract GPU detection into shared functions**

Move `is_nvidia_gpu()`, `is_amd_gpu()`, `is_intel_gpu()` from `main.rs` to a shared module (e.g., `src-tauri/src/config/gpu_detection.rs`) so both `main.rs` and `lib.rs` can use them without duplication.

- [ ] **Step 4: Verify compilation**

Run: `cd src-tauri && cargo check`

- [ ] **Step 5: Test scenarios**

Manual verification:
1. With current setup (Intel + NVIDIA): should behave exactly as before (existing config is not default)
2. Fresh install simulation: delete `graphics_settings.db`, restart — should detect GPU and apply smart defaults

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/lib.rs src-tauri/src/main.rs \
  src-tauri/src/config/graphics_settings.rs \
  src-tauri/src/config/gpu_detection.rs \
  src-tauri/src/config/mod.rs
git commit -m "feat: GPU-aware smart defaults for unconfigured users

NVIDIA-only Wayland users get compositing enabled by default.
Only applies when graphics settings are at DB defaults (never
configured). Existing user configs are never modified.

cc"
```

---

## Phase 3: First-Run Setup Wizard

### Task 3.1: Backend — First-run detection and completion tracking

**Files:**
- Create: `src-tauri/src/config/setup_wizard.rs`
- Modify: `src-tauri/src/config/mod.rs`
- Modify: `src-tauri/src/commands_v2.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Create setup_wizard.rs**

Simple SQLite store tracking wizard completion:

```rust
pub struct SetupWizardStore { conn: Connection }

impl SetupWizardStore {
    pub fn new() -> Result<Self, String> { /* data_dir/qbz/setup_wizard.db */ }
    pub fn is_completed(&self) -> Result<bool, String> { /* SELECT completed */ }
    pub fn mark_completed(&self) -> Result<(), String> { /* UPDATE completed = 1 */ }
    pub fn reset(&self) -> Result<(), String> { /* UPDATE completed = 0 (for dev re-run) */ }
}
```

- [ ] **Step 2: Add V2 commands**

```rust
v2_is_setup_wizard_completed() -> Result<bool, String>
v2_mark_setup_wizard_completed() -> Result<(), String>
v2_reset_setup_wizard() -> Result<(), String>  // for Developer Mode "Re-run wizard" button
```

- [ ] **Step 3: Register and verify**

- [ ] **Step 4: Commit**

---

### Task 3.2: Frontend — Wizard component

**Files:**
- Create: `src/lib/components/SetupWizard.svelte`
- Modify: `src/routes/+page.svelte` (show wizard on first run)
- Modify: `src/lib/i18n/locales/*.json` (5 files)

- [ ] **Step 1: Design wizard steps**

The wizard is a full-screen modal (using existing Modal component with `maxWidth="680px"`) with a step indicator at the top (like Cider).

**Steps:**

1. **Welcome + Language**
   - QBZ logo
   - "Welcome to QBZ" / subtitle
   - Language dropdown (en/es/de/fr/pt)
   - Next button

2. **Audio Setup**
   - Auto-detect available backends (call `v2_get_available_backends`)
   - Auto-detect devices (call `v2_get_devices_for_backend`)
   - Select output device
   - Toggle: Exclusive Mode (with explanation)
   - Toggle: DAC Passthrough (with explanation)
   - "You can change these later in Settings > Audio"

3. **Rendering Setup**
   - Auto-detect GPU (show detected: NVIDIA/Intel/AMD)
   - Auto-detect session (Wayland/X11)
   - Recommended preset based on detection (auto-selected):
     - NVIDIA-only Wayland: "Recommended: Hardware Acceleration ON, DMA-BUF OFF"
     - Intel/AMD: "Recommended: Software rendering (most stable)"
     - Hybrid: "Recommended: Software rendering (iGPU handles it)"
   - User can override
   - Preview: "Your config will set these env vars: ..."

4. **Login**
   - Qobuz email + password (reuse LoginView logic)
   - Or "Skip — log in later"

5. **Done**
   - Summary of choices
   - "Start listening" button
   - Calls `v2_mark_setup_wizard_completed()`

- [ ] **Step 2: Implement SetupWizard.svelte**

Use existing patterns:
- Step state: `let currentStep = $state(0)`
- Step indicator: row of circles with connecting lines (like Cider)
- Each step is a `{#if currentStep === N}` block
- Previous/Next buttons in footer
- Calls `invoke()` for backend detection and settings save
- On completion: calls `v2_mark_setup_wizard_completed()` + `onComplete` callback

- [ ] **Step 3: Add i18n keys**

Under `setupWizard` namespace in all 5 locale files. ~30-40 keys covering step titles, descriptions, tooltips, button labels.

- [ ] **Step 4: Integrate in +page.svelte**

In the `decideLaunchModals()` flow or after login:

```typescript
// After successful login, check if wizard has been completed
const wizardCompleted = await invoke<boolean>('v2_is_setup_wizard_completed');
if (!wizardCompleted) {
    showSetupWizard = true;
}
```

The wizard shows AFTER login (needs auth for some features) but BEFORE the main app.

- [ ] **Step 5: Add "Re-run Setup Wizard" button to Developer Mode**

In SettingsView.svelte Developer section, add a button that calls `v2_reset_setup_wizard()` and shows a toast.

- [ ] **Step 6: Verify with svelte-check**

- [ ] **Step 7: Commit**

```bash
git add src/lib/components/SetupWizard.svelte \
  src/routes/+page.svelte \
  src/lib/i18n/locales/*.json \
  src/lib/components/views/SettingsView.svelte
git commit -m "feat: first-run setup wizard

Multi-step wizard for language, audio, rendering, and login.
Auto-detects GPU and audio hardware for smart recommendations.
Only shows on first launch; can be re-run from Developer Mode.

cc"
```

---

## Dependencies Between Phases

```
Phase 1 (Diagnostics) ← independent, can ship alone
Phase 2 (NVIDIA defaults) ← independent, can ship alone
Phase 3 (Setup Wizard) ← benefits from Phase 2 (GPU detection), but not blocked
```

All phases can be worked on in parallel or sequentially. Phase 1 is most immediately useful for debugging the current rendering issues.
