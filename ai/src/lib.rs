// FreeSynergy AI Manager — backend
//
// Responsibilities:
//   - AiEngine trait: common interface for all AI engines
//   - LlmEngine: mistral.rs-backed LLM server (start, stop, status, PID management)
//   - LlmModel: predefined model catalogue (Qwen3-4B, Qwen3-8B, Qwen2.5-Coder-7B)
//   - write_continue_config: writes ~/.continue/config.json for editor integration
//
// Process management: PID-file based — write on start, read/kill on stop.
// Alive check: /proc/{pid} on Linux, `kill -0` otherwise.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

// ── EngineType ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EngineType {
    Llm,
    // ImageGen, Speech, Embedding — future
}

// ── EngineStatus ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum EngineStatus {
    Stopped,
    Running { port: u16 },
    Error(String),
}

impl EngineStatus {
    pub fn is_running(&self) -> bool {
        matches!(self, Self::Running { .. })
    }

    pub fn label(&self) -> &str {
        match self {
            Self::Stopped => "Stopped",
            Self::Running { .. } => "Running",
            Self::Error(_) => "Error",
        }
    }
}

// ── ModelSpec ─────────────────────────────────────────────────────────────────

/// Static data record for one predefined LLM model.
///
/// Adding a new model means adding one entry to [`CATALOGUE`] — no other
/// code needs to change.
pub struct ModelSpec {
    /// Hugging Face model ID, e.g. `"Qwen/Qwen3-4B"`.
    pub hf_id: &'static str,
    /// Human-readable name shown in UI pickers.
    pub display_name: &'static str,
    /// Estimated RAM in GB after ISQ Q4K quantization.
    pub ram_gb: f32,
    /// Factory: construct the matching [`LlmModel`] variant.
    make: fn() -> LlmModel,
}

/// All built-in models. Order determines `all_predefined()` output.
///
/// To add a new model: add one row here + one variant to `LlmModel` + one arm
/// in `spec()`.  Everything else (display_name, ram_gb, from_hf_id, …) is
/// driven automatically from this table.
const CATALOGUE: &[ModelSpec] = &[
    ModelSpec {
        hf_id: "Qwen/Qwen3-4B",
        display_name: "Qwen3-4B  (~3.5 GB RAM, fast)",
        ram_gb: 3.5,
        make: || LlmModel::Qwen3_4B,
    },
    ModelSpec {
        hf_id: "Qwen/Qwen3-8B",
        display_name: "Qwen3-8B  (~6 GB RAM, better quality)",
        ram_gb: 6.0,
        make: || LlmModel::Qwen3_8B,
    },
    ModelSpec {
        hf_id: "Qwen/Qwen2.5-Coder-7B",
        display_name: "Qwen2.5-Coder-7B  (~5 GB RAM, code-focused)",
        ram_gb: 5.0,
        make: || LlmModel::Qwen25Coder7B,
    },
];

// ── LlmModel ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LlmModel {
    Qwen3_4B,
    Qwen3_8B,
    Qwen25Coder7B,
    Custom(String),
}

impl LlmModel {
    /// Look up the static [`ModelSpec`] for predefined variants.
    ///
    /// This is the single match block that drives all other methods.
    /// Custom models return `None`.
    fn spec(&self) -> Option<&'static ModelSpec> {
        match self {
            Self::Qwen3_4B => Some(&CATALOGUE[0]),
            Self::Qwen3_8B => Some(&CATALOGUE[1]),
            Self::Qwen25Coder7B => Some(&CATALOGUE[2]),
            Self::Custom(_) => None,
        }
    }

    /// Hugging Face model ID (e.g. `"Qwen/Qwen3-4B"`).
    pub fn hf_id(&self) -> &str {
        self.spec().map_or_else(
            || {
                if let Self::Custom(id) = self {
                    id.as_str()
                } else {
                    unreachable!()
                }
            },
            |s| s.hf_id,
        )
    }

    /// Human-readable name for UI pickers.
    pub fn display_name(&self) -> &str {
        self.spec().map_or("Custom model", |s| s.display_name)
    }

    /// Estimated RAM in GB after ISQ Q4K quantization.
    pub fn ram_gb(&self) -> f32 {
        self.spec().map_or(0.0, |s| s.ram_gb)
    }

    /// All predefined (non-custom) models in catalogue order.
    pub fn all_predefined() -> impl Iterator<Item = LlmModel> {
        CATALOGUE.iter().map(|s| (s.make)())
    }

    /// Construct from a Hugging Face ID; falls back to `Custom`.
    pub fn from_hf_id(id: &str) -> Self {
        CATALOGUE
            .iter()
            .find(|s| s.hf_id == id)
            .map(|s| (s.make)())
            .unwrap_or_else(|| Self::Custom(id.into()))
    }
}

// ── LlmConfig ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    pub model: LlmModel,
    pub port: u16,
    pub host: String,
    pub isq: String,
    pub max_seqs: u32,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            model: LlmModel::Qwen3_4B,
            port: 1234,
            host: "127.0.0.1".into(),
            isq: "q4k".into(),
            max_seqs: 4,
        }
    }
}

// ── AiEngine trait ────────────────────────────────────────────────────────────

pub trait AiEngine {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn engine_type(&self) -> EngineType;
    fn status(&self) -> EngineStatus;
    fn start(&self) -> Result<(), AiError>;
    fn stop(&self) -> Result<(), AiError>;
}

// ── LlmEngine ────────────────────────────────────────────────────────────────

/// LLM inference engine backed by mistral.rs (`mistralrs serve`).
pub struct LlmEngine {
    pub config: LlmConfig,
    pub binary_path: PathBuf,
    pub data_dir: PathBuf,
}

impl LlmEngine {
    pub fn new(
        config: LlmConfig,
        binary_path: impl Into<PathBuf>,
        data_dir: impl Into<PathBuf>,
    ) -> Self {
        Self {
            config,
            binary_path: binary_path.into(),
            data_dir: data_dir.into(),
        }
    }

    /// Default install path for the mistral.rs binary.
    pub fn default_binary() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/root".into());
        PathBuf::from(home).join(".local/share/fsn/bin/mistral/mistralrs")
    }

    /// Default data directory for logs, PID file, and model cache.
    pub fn default_data_dir() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/root".into());
        PathBuf::from(home).join(".local/share/fsn/data/mistral")
    }

    fn pid_file(&self) -> PathBuf {
        self.data_dir.join("mistral.pid")
    }

    fn log_file(&self) -> PathBuf {
        self.data_dir.join("mistral.log")
    }

    fn read_pid(&self) -> Option<u32> {
        std::fs::read_to_string(self.pid_file())
            .ok()
            .and_then(|s| s.trim().parse().ok())
    }

    fn is_pid_alive(pid: u32) -> bool {
        // Linux: /proc/{pid} exists while the process is alive.
        #[cfg(target_os = "linux")]
        return std::path::Path::new(&format!("/proc/{pid}")).exists();

        // Other Unix: send signal 0 (no-op) — succeeds if process exists.
        #[cfg(all(unix, not(target_os = "linux")))]
        return std::process::Command::new("kill")
            .args(["-0", &pid.to_string()])
            .status()
            .map(|s| s.success())
            .unwrap_or(false);

        // Windows: not supported yet.
        #[cfg(windows)]
        return false;
    }

    pub fn is_installed(&self) -> bool {
        self.binary_path.exists()
    }

    /// Writes `~/.continue/config.json` so the editor can use the local LLM.
    /// Call after a successful `start()`.
    pub fn write_continue_config(&self) -> Result<(), AiError> {
        let home = std::env::var("HOME").map_err(|_| AiError::Config("HOME not set".into()))?;
        let continue_dir = PathBuf::from(home).join(".continue");

        std::fs::create_dir_all(&continue_dir).map_err(|e| AiError::Io(e.to_string()))?;

        let api_base = format!("http://{}:{}/v1", self.config.host, self.config.port);

        let system_prompt = "/no_think\n\n\
            You are a senior Rust engineer and coding assistant for the FreeSynergy project.\n\n\
            FreeSynergy is a self-hosted platform written in Rust. Tech stack:\n\
            - Rust (all services and CLIs)\n\
            - Dioxus (desktop UI, WebView)\n\
            - SQLite (6 databases per node)\n\
            - S3-compatible storage\n\
            - TOML for config/manifests\n\n\
            Rules: code and comments in English, chat in German, concise answers, \
            OOP with traits over match blocks. \
            Complex cross-repo architecture → Claude Code.";

        let config = serde_json::json!({
            "models": [{
                "title": format!("{} (lokal)", self.config.model.hf_id().split('/').last().unwrap_or("LLM")),
                "provider": "openai",
                "model": "default",
                "apiBase": api_base,
                "apiKey": "none",
                "systemMessage": system_prompt,
                "completionOptions": { "temperature": 0.2, "maxTokens": 1024 }
            }],
            "tabAutocompleteModel": {
                "title": "Autocomplete (lokal)",
                "provider": "openai",
                "model": "default",
                "apiBase": api_base,
                "apiKey": "none"
            },
            "allowAnonymousTelemetry": false
        });

        let json =
            serde_json::to_string_pretty(&config).map_err(|e| AiError::Config(e.to_string()))?;

        std::fs::write(continue_dir.join("config.json"), json)
            .map_err(|e| AiError::Io(e.to_string()))?;

        Ok(())
    }
}

impl AiEngine for LlmEngine {
    fn id(&self) -> &str {
        "mistral"
    }
    fn name(&self) -> &str {
        "Mistral.rs"
    }
    fn engine_type(&self) -> EngineType {
        EngineType::Llm
    }

    fn status(&self) -> EngineStatus {
        let Some(pid) = self.read_pid() else {
            return EngineStatus::Stopped;
        };
        if Self::is_pid_alive(pid) {
            EngineStatus::Running {
                port: self.config.port,
            }
        } else {
            let _ = std::fs::remove_file(self.pid_file()); // clean up stale PID
            EngineStatus::Stopped
        }
    }

    fn start(&self) -> Result<(), AiError> {
        if self.status().is_running() {
            return Ok(());
        }
        if !self.is_installed() {
            return Err(AiError::NotInstalled(
                self.binary_path.display().to_string(),
            ));
        }

        std::fs::create_dir_all(&self.data_dir).map_err(|e| AiError::Io(e.to_string()))?;

        let log = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.log_file())
            .map_err(|e| AiError::Io(e.to_string()))?;

        let mut cmd = std::process::Command::new(&self.binary_path);
        cmd.arg("serve")
            .arg("--port")
            .arg(self.config.port.to_string())
            .arg("--host")
            .arg(&self.config.host)
            .arg("--max-seqs")
            .arg(self.config.max_seqs.to_string())
            .arg("-m")
            .arg(self.config.model.hf_id());

        if !self.config.isq.is_empty() {
            cmd.arg("--isq").arg(&self.config.isq);
        }

        let child = cmd
            .stdout(log.try_clone().map_err(|e| AiError::Io(e.to_string()))?)
            .stderr(log)
            .spawn()
            .map_err(|e| AiError::SpawnFailed(e.to_string()))?;

        let pid = child.id();
        std::mem::forget(child); // detach — let the process outlive this handle

        std::fs::write(self.pid_file(), pid.to_string()).map_err(|e| AiError::Io(e.to_string()))?;

        Ok(())
    }

    fn stop(&self) -> Result<(), AiError> {
        let Some(pid) = self.read_pid() else {
            return Ok(()); // already stopped
        };

        std::process::Command::new("kill")
            .arg(pid.to_string())
            .status()
            .map_err(|e| AiError::Io(e.to_string()))?;

        let _ = std::fs::remove_file(self.pid_file());
        Ok(())
    }
}

// ── AiError ───────────────────────────────────────────────────────────────────

#[derive(Debug, thiserror::Error)]
pub enum AiError {
    #[error("IO error: {0}")]
    Io(String),
    #[error("Failed to spawn process: {0}")]
    SpawnFailed(String),
    #[error("Config error: {0}")]
    Config(String),
    #[error("Binary not installed at: {0}")]
    NotInstalled(String),
}
