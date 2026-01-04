// Embedded LLM module - Offline model inference with GPU + Quantization support
use anyhow::{anyhow, Result};
use candle_core::{DType, Device, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::generation::LogitsProcessor;
use candle_transformers::models::qwen2::{Config, ModelForCausalLM as Qwen2};
use hf_hub::{api::sync::Api, Repo, RepoType};
use std::fs;
use std::path::PathBuf;
use tokenizers::Tokenizer;

const MAX_TOKENS: usize = 512;

/// Device type for inference
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DeviceType {
    Cpu,
    Cuda(usize), // GPU index
    Metal,
}

/// Quantization level
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Quantization {
    F32, // Full precision (default)
    F16, // Half precision
    BF16, // BFloat16
         // Note: INT8/INT4 require GGUF format, not yet supported
}

/// Detect best available device
pub fn detect_device() -> DeviceType {
    // Try CUDA first
    #[cfg(feature = "cuda")]
    {
        if let Ok(_) = Device::cuda(0) {
            return DeviceType::Cuda(0);
        }
    }

    // Try Metal (macOS)
    #[cfg(feature = "metal")]
    {
        if let Ok(_) = Device::new_metal(0) {
            return DeviceType::Metal;
        }
    }

    DeviceType::Cpu
}

/// Get device from DeviceType
pub fn get_device(device_type: DeviceType) -> Result<Device> {
    match device_type {
        DeviceType::Cpu => Ok(Device::Cpu),
        DeviceType::Cuda(idx) => {
            #[cfg(feature = "cuda")]
            {
                Device::cuda(idx).map_err(|e| anyhow!("CUDA error: {}", e))
            }
            #[cfg(not(feature = "cuda"))]
            {
                let _ = idx;
                Err(anyhow!(
                    "CUDA not enabled. Rebuild with: cargo build --features cuda"
                ))
            }
        }
        DeviceType::Metal => {
            #[cfg(feature = "metal")]
            {
                Device::new_metal(0).map_err(|e| anyhow!("Metal error: {}", e))
            }
            #[cfg(not(feature = "metal"))]
            {
                Err(anyhow!(
                    "Metal not enabled. Rebuild with: cargo build --features metal"
                ))
            }
        }
    }
}

/// Get DType from Quantization
pub fn get_dtype(quant: Quantization, device: &Device) -> DType {
    match quant {
        Quantization::F32 => DType::F32,
        Quantization::F16 => DType::F16,
        Quantization::BF16 => {
            // BF16 not supported on all devices
            if device.is_cpu() {
                DType::F32 // Fallback
            } else {
                DType::BF16
            }
        }
    }
}

/// Get device info string
pub fn device_info() -> String {
    let device_type = detect_device();
    match device_type {
        DeviceType::Cpu => "CPU".to_string(),
        DeviceType::Cuda(idx) => format!("CUDA GPU {}", idx),
        DeviceType::Metal => "Metal GPU".to_string(),
    }
}

/// Available models for download
#[derive(Debug, Clone)]
pub struct ModelInfo {
    pub id: &'static str,
    pub name: &'static str,
    pub hf_repo: &'static str,
    pub size_gb: f32,
    pub params: &'static str,
    pub description: &'static str,
}

pub const AVAILABLE_MODELS: &[ModelInfo] = &[
    ModelInfo {
        id: "qwen-coder-0.5b",
        name: "Qwen2.5-Coder-0.5B",
        hf_repo: "Qwen/Qwen2.5-Coder-0.5B-Instruct",
        size_gb: 1.0,
        params: "0.5B",
        description: "Smallest code model, fast inference",
    },
    ModelInfo {
        id: "qwen-coder-1.5b",
        name: "Qwen2.5-Coder-1.5B",
        hf_repo: "Qwen/Qwen2.5-Coder-1.5B-Instruct",
        size_gb: 3.0,
        params: "1.5B",
        description: "Best balance of size and code quality (recommended)",
    },
    ModelInfo {
        id: "qwen-coder-3b",
        name: "Qwen2.5-Coder-3B",
        hf_repo: "Qwen/Qwen2.5-Coder-3B-Instruct",
        size_gb: 6.0,
        params: "3B",
        description: "High quality code understanding",
    },
    ModelInfo {
        id: "deepseek-coder-1.3b",
        name: "DeepSeek-Coder-1.3B",
        hf_repo: "deepseek-ai/deepseek-coder-1.3b-instruct",
        size_gb: 2.6,
        params: "1.3B",
        description: "Excellent code model, efficient",
    },
    ModelInfo {
        id: "qwen-0.5b",
        name: "Qwen2.5-0.5B",
        hf_repo: "Qwen/Qwen2.5-0.5B-Instruct",
        size_gb: 1.0,
        params: "0.5B",
        description: "General purpose, very fast",
    },
];

/// Get model info by ID
pub fn get_model_info(id: &str) -> Option<&'static ModelInfo> {
    AVAILABLE_MODELS.iter().find(|m| m.id == id)
}

/// Get the config directory for vibecheck
pub fn get_config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("vibecheck")
}

/// Get the models cache directory
pub fn get_models_dir() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("vibecheck")
        .join("models")
}

/// Get the current model setting
pub fn get_current_model() -> Option<String> {
    let config_path = get_config_dir().join("current_model");
    fs::read_to_string(config_path).ok()
}

/// Set the current model
pub fn set_current_model(model_id: &str) -> Result<()> {
    let config_dir = get_config_dir();
    fs::create_dir_all(&config_dir)?;
    fs::write(config_dir.join("current_model"), model_id)?;
    Ok(())
}

/// Check which models are downloaded
pub fn get_downloaded_models() -> Vec<String> {
    let models_dir = get_models_dir();
    let mut downloaded = Vec::new();

    for model in AVAILABLE_MODELS {
        let model_dir = models_dir.join(model.id);
        if model_dir.join("config.json").exists() {
            downloaded.push(model.id.to_string());
        }
    }

    downloaded
}

/// Download a model
pub fn download_model(model_id: &str) -> Result<PathBuf> {
    let model_info =
        get_model_info(model_id).ok_or_else(|| anyhow!("Unknown model: {}", model_id))?;

    println!(
        "Downloading {} (~{:.1}GB)...",
        model_info.name, model_info.size_gb
    );
    println!("  From: {}", model_info.hf_repo);

    let api = Api::new()?;
    let repo = api.repo(Repo::new(model_info.hf_repo.to_string(), RepoType::Model));

    // Download required files
    println!("  Fetching tokenizer.json...");
    let _ = repo.get("tokenizer.json")?;

    println!("  Fetching config.json...");
    let _ = repo.get("config.json")?;

    println!("  Fetching model weights (this may take a while)...");

    // Try single file first, then sharded
    if repo.get("model.safetensors").is_err() {
        // Try sharded weights
        for i in 1..=10 {
            let name = format!("model-{:05}-of-{:05}.safetensors", i, 2);
            if repo.get(&name).is_err() {
                break;
            }
            println!("    Downloaded shard {}", i);
        }
    }

    // Mark as downloaded in our models dir
    let model_dir = get_models_dir().join(model_id);
    fs::create_dir_all(&model_dir)?;
    fs::write(model_dir.join("config.json"), model_info.hf_repo)?;

    println!("  {} downloaded successfully!", model_info.name);

    Ok(model_dir)
}

/// List all models with download status
pub fn list_models() {
    let downloaded = get_downloaded_models();
    let current = get_current_model();

    println!("\nAvailable Models:\n");
    println!(
        "{:<20} {:<8} {:<8} {:<10} Description",
        "ID", "Params", "Size", "Status"
    );
    println!("{}", "-".repeat(80));

    for model in AVAILABLE_MODELS {
        let status = if downloaded.contains(&model.id.to_string()) {
            if current.as_deref() == Some(model.id) {
                "active"
            } else {
                "ready"
            }
        } else {
            "not downloaded"
        };

        println!(
            "{:<20} {:<8} {:<8} {:<10} {}",
            model.id,
            model.params,
            format!("~{}GB", model.size_gb),
            status,
            model.description
        );
    }

    println!("\nCommands:");
    println!("  vibecheck models download <id>  - Download a model");
    println!("  vibecheck models use <id>       - Switch to a model");
    println!("  vibecheck models remove <id>    - Remove a downloaded model");
}

pub struct EmbeddedLlm {
    model: Qwen2,
    tokenizer: Tokenizer,
    device: Device,
    #[allow(dead_code)]
    config: Config,
    context: String,
    model_name: String,
}

impl EmbeddedLlm {
    /// Initialize with a specific model
    pub fn new(model_id: Option<&str>) -> Result<Self> {
        Self::new_with_options(model_id, None, None)
    }

    /// Initialize with specific device and quantization options
    pub fn new_with_options(
        model_id: Option<&str>,
        device_type: Option<DeviceType>,
        quantization: Option<Quantization>,
    ) -> Result<Self> {
        // Determine which model to use
        let model_id = model_id
            .map(String::from)
            .or_else(get_current_model)
            .unwrap_or_else(|| "qwen-coder-1.5b".to_string());

        let model_info = get_model_info(&model_id).ok_or_else(|| {
            anyhow!(
                "Unknown model: {}. Run 'vibecheck models' to see available models.",
                model_id
            )
        })?;

        // Check if downloaded
        let downloaded = get_downloaded_models();
        if !downloaded.contains(&model_id) {
            return Err(anyhow!(
                "Model '{}' not downloaded. Run: vibecheck models download {}",
                model_id,
                model_id
            ));
        }

        // Auto-detect device or use specified
        let device_type = device_type.unwrap_or_else(detect_device);
        let device = get_device(device_type)?;

        // Default quantization based on device
        let quant = quantization.unwrap_or({
            match device_type {
                DeviceType::Cpu => Quantization::F32,
                DeviceType::Cuda(_) => Quantization::F16, // GPU benefits from F16
                DeviceType::Metal => Quantization::F16,
            }
        });
        let dtype = get_dtype(quant, &device);

        println!("Loading {} ({})...", model_info.name, model_info.params);
        println!("  Device: {:?}", device_type);
        println!("  Precision: {:?}", quant);

        // Load from HuggingFace cache
        let api = Api::new()?;
        let repo = api.repo(Repo::new(model_info.hf_repo.to_string(), RepoType::Model));

        let tokenizer_path = repo.get("tokenizer.json")?;
        let config_path = repo.get("config.json")?;

        // Load config
        let config_str = fs::read_to_string(&config_path)?;
        let config: Config = serde_json::from_str(&config_str)?;

        // Load tokenizer
        let tokenizer = Tokenizer::from_file(&tokenizer_path)
            .map_err(|e| anyhow!("Failed to load tokenizer: {}", e))?;

        // Load model weights
        let weights_files: Vec<PathBuf> = if let Ok(path) = repo.get("model.safetensors") {
            vec![path]
        } else {
            let mut files = Vec::new();
            for i in 1..=10 {
                let name = format!("model-{:05}-of-{:05}.safetensors", i, 2);
                if let Ok(path) = repo.get(&name) {
                    files.push(path);
                } else {
                    break;
                }
            }
            files
        };

        if weights_files.is_empty() {
            return Err(anyhow!("No model weights found"));
        }

        let vb = unsafe { VarBuilder::from_mmaped_safetensors(&weights_files, dtype, &device)? };

        println!("  Building model...");
        let model = Qwen2::new(&config, vb)?;

        println!("  {} ready!", model_info.name);

        // Set as current model
        set_current_model(&model_id)?;

        Ok(Self {
            model,
            tokenizer,
            device,
            config,
            context: String::new(),
            model_name: model_info.name.to_string(),
        })
    }

    /// Set the analysis context
    pub fn set_context(&mut self, context: &str) {
        self.context = context.to_string();
    }

    /// Get model name
    pub fn model_name(&self) -> &str {
        &self.model_name
    }

    /// Generate a response
    pub fn generate(&mut self, prompt: &str) -> Result<String> {
        // Clear KV cache to reset model state for new query
        self.model.clear_kv_cache();

        let full_prompt = format!(
            "<|im_start|>system\nYou are vibecheck, a helpful AI that analyzes coding tool usage. Be concise and actionable.<|im_end|>\n<|im_start|>user\nContext:\n{}\n\nQuestion: {}<|im_end|>\n<|im_start|>assistant\n",
            self.context, prompt
        );

        let tokens = self
            .tokenizer
            .encode(full_prompt.as_str(), true)
            .map_err(|e| anyhow!("Tokenization error: {}", e))?;

        let mut token_ids: Vec<u32> = tokens.get_ids().to_vec();
        let mut generated = String::new();

        let mut logits_processor = LogitsProcessor::new(42, Some(0.7), Some(0.9));

        let eos_token = self
            .tokenizer
            .token_to_id("<|im_end|>")
            .or_else(|| self.tokenizer.token_to_id("<|endoftext|>"))
            .unwrap_or(151643);

        let mut pos = 0;

        for _ in 0..MAX_TOKENS {
            let input = Tensor::new(&token_ids[pos..], &self.device)?.unsqueeze(0)?;
            let logits = self.model.forward(&input, pos)?;

            let logits = logits.squeeze(0)?;
            let logits = logits.get(logits.dim(0)? - 1)?;

            let next_token = logits_processor.sample(&logits)?;

            if next_token == eos_token {
                break;
            }

            pos = token_ids.len();
            token_ids.push(next_token);

            if let Ok(text) = self.tokenizer.decode(&[next_token], false) {
                generated.push_str(&text);
            }
        }

        Ok(generated.trim().to_string())
    }

    /// Analyze data and provide insights
    pub fn analyze(&mut self) -> Result<String> {
        self.generate(
            "Analyze this AI tool usage data. What are the top 3 actionable recommendations to optimize storage and workflow?"
        )
    }

    /// Get recommendations for a specific topic
    pub fn get_recommendations(&mut self, topic: &str) -> Result<String> {
        self.generate(&format!("Give me specific recommendations for: {}", topic))
    }
}
