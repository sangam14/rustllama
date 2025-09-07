/*!
# YAML Configuration Support for RustLama

This module provides YAML configuration file support, allowing users to define
complex inference tasks, model management operations, and batch processing
through declarative configuration files.

## Example YAML Configuration

```yaml
# rustlama.yml
version: "1.0"
name: "My LLaMA Tasks"
description: "Batch inference and model management"

# Default settings
defaults:
  model: "TheBloke/Llama-2-7B-Chat-GGUF"
  max_tokens: 1024
  temperature: 0.8
  verbose: true

# Model management tasks
models:
  - action: "pull"
    model_id: "TheBloke/Llama-2-7B-Chat-GGUF"
    filename: "llama-2-7b-chat.Q4_K_M.gguf"
    force: false
  
  - action: "pull" 
    model_id: "TheBloke/Mistral-7B-Instruct-v0.1-GGUF"
    filename: "mistral-7b-instruct-v0.1.Q4_K_M.gguf"

# Inference tasks
tasks:
  - name: "Creative Writing"
    prompt: "Write a short story about space exploration"
    model: "TheBloke/Llama-2-7B-Chat-GGUF"
    max_tokens: 512
    temperature: 1.0
    output_file: "creative_story.txt"
  
  - name: "Technical Documentation" 
    prompt: "Explain how neural networks work"
    model: "TheBloke/Llama-2-7B-Chat-GGUF"
    max_tokens: 1024
    temperature: 0.3
    output_file: "neural_networks_explanation.txt"
```
*/

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Main YAML configuration structure
#[derive(Debug, Serialize, Deserialize)]
pub struct YamlConfig {
    /// Configuration version
    pub version: String,
    
    /// Configuration name/title
    #[serde(default)]
    pub name: Option<String>,
    
    /// Configuration description
    #[serde(default)]
    pub description: Option<String>,
    
    /// Default settings for all tasks
    #[serde(default)]
    pub defaults: Option<DefaultConfig>,
    
    /// Model management operations
    #[serde(default)]
    pub models: Vec<ModelTask>,
    
    /// Inference tasks
    #[serde(default)]
    pub tasks: Vec<InferenceTask>,
    
    /// Environment variables
    #[serde(default)]
    pub environment: HashMap<String, String>,
}

/// Default configuration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefaultConfig {
    /// Default model to use
    #[serde(default)]
    pub model: Option<String>,
    
    /// Default HuggingFace filename
    #[serde(default)]
    pub hf_filename: Option<String>,
    
    /// Default cache directory
    #[serde(default)]
    pub cache_dir: Option<String>,
    
    /// Default maximum tokens
    #[serde(default)]
    pub max_tokens: Option<usize>,
    
    /// Default temperature
    #[serde(default)]
    pub temperature: Option<f32>,
    
    /// Default top-k
    #[serde(default)]
    pub top_k: Option<usize>,
    
    /// Default top-p
    #[serde(default)]
    pub top_p: Option<f32>,
    
    /// Default context size
    #[serde(default)]
    pub ctx_size: Option<u32>,
    
    /// Default thread count
    #[serde(default)]
    pub threads: Option<i32>,
    
    /// Default verbose setting
    #[serde(default)]
    pub verbose: Option<bool>,
    
    /// Default no-color setting
    #[serde(default)]
    pub no_color: Option<bool>,
    
    /// Default stats setting
    #[serde(default)]
    pub stats: Option<bool>,
}

/// Model management task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelTask {
    /// Action to perform: "pull", "remove", "list", "usage"
    pub action: String,
    
    /// Model ID for pull/remove actions
    #[serde(default)]
    pub model_id: Option<String>,
    
    /// Specific filename to download
    #[serde(default)]
    pub filename: Option<String>,
    
    /// Cache directory
    #[serde(default)]
    pub cache_dir: Option<String>,
    
    /// Force operation
    #[serde(default)]
    pub force: bool,
    
    /// Verbose output
    #[serde(default)]
    pub verbose: bool,
    
    /// Task description
    #[serde(default)]
    pub description: Option<String>,
}

/// Inference task configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceTask {
    /// Task name
    pub name: String,
    
    /// Input prompt
    pub prompt: String,
    
    /// Model to use (overrides default)
    #[serde(default)]
    pub model: Option<String>,
    
    /// HuggingFace filename
    #[serde(default)]
    pub hf_filename: Option<String>,
    
    /// Cache directory
    #[serde(default)]
    pub cache_dir: Option<String>,
    
    /// Force download
    #[serde(default)]
    pub force_download: bool,
    
    /// Maximum tokens to generate
    #[serde(default)]
    pub max_tokens: Option<usize>,
    
    /// Sampling temperature
    #[serde(default)]
    pub temperature: Option<f32>,
    
    /// Top-k sampling
    #[serde(default)]
    pub top_k: Option<usize>,
    
    /// Top-p sampling
    #[serde(default)]
    pub top_p: Option<f32>,
    
    /// Context size
    #[serde(default)]
    pub ctx_size: Option<u32>,
    
    /// Number of threads
    #[serde(default)]
    pub threads: Option<i32>,
    
    /// Disable colored output
    #[serde(default)]
    pub no_color: bool,
    
    /// Show statistics
    #[serde(default)]
    pub stats: bool,
    
    /// Verbose output
    #[serde(default)]
    pub verbose: bool,
    
    /// Output file path (optional)
    #[serde(default)]
    pub output_file: Option<String>,
    
    /// Task description
    #[serde(default)]
    pub description: Option<String>,
    
    /// Continue on error for batch processing
    #[serde(default)]
    pub continue_on_error: bool,
}

impl YamlConfig {
    /// Load configuration from YAML file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(&path)
            .map_err(|e| anyhow!("Failed to read YAML file '{}': {}", path.as_ref().display(), e))?;
        
        let config: YamlConfig = serde_yaml::from_str(&content)
            .map_err(|e| anyhow!("Failed to parse YAML configuration: {}", e))?;
        
        // Validate configuration
        config.validate()?;
        
        Ok(config)
    }
    
    /// Save configuration to YAML file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = serde_yaml::to_string(self)
            .map_err(|e| anyhow!("Failed to serialize configuration: {}", e))?;
        
        fs::write(&path, content)
            .map_err(|e| anyhow!("Failed to write YAML file '{}': {}", path.as_ref().display(), e))?;
        
        Ok(())
    }
    
    /// Validate the configuration
    pub fn validate(&self) -> Result<()> {
        // Check version
        if self.version.is_empty() {
            return Err(anyhow!("Configuration version is required"));
        }
        
        // Validate model tasks
        for (i, model_task) in self.models.iter().enumerate() {
            if !["pull", "remove", "list", "usage"].contains(&model_task.action.as_str()) {
                return Err(anyhow!(
                    "Invalid model action '{}' in task {}: must be one of: pull, remove, list, usage", 
                    model_task.action, i
                ));
            }
            
            // Pull and remove actions require model_id
            if matches!(model_task.action.as_str(), "pull" | "remove") && model_task.model_id.is_none() {
                return Err(anyhow!(
                    "Model action '{}' in task {} requires model_id",
                    model_task.action, i
                ));
            }
        }
        
        // Validate inference tasks
        for (i, task) in self.tasks.iter().enumerate() {
            if task.name.is_empty() {
                return Err(anyhow!("Task {} must have a name", i));
            }
            
            if task.prompt.is_empty() {
                return Err(anyhow!("Task '{}' must have a prompt", task.name));
            }
            
            // Validate parameter ranges
            if let Some(temp) = task.temperature {
                if temp < 0.0 || temp > 2.0 {
                    return Err(anyhow!(
                        "Task '{}': temperature must be between 0.0 and 2.0", 
                        task.name
                    ));
                }
            }
            
            if let Some(top_p) = task.top_p {
                if top_p < 0.0 || top_p > 1.0 {
                    return Err(anyhow!(
                        "Task '{}': top_p must be between 0.0 and 1.0", 
                        task.name
                    ));
                }
            }
        }
        
        Ok(())
    }
    
    /// Apply defaults to an inference task
    pub fn apply_defaults(&self, task: &mut InferenceTask) {
        if let Some(defaults) = &self.defaults {
            if task.model.is_none() {
                task.model = defaults.model.clone();
            }
            if task.hf_filename.is_none() {
                task.hf_filename = defaults.hf_filename.clone();
            }
            if task.cache_dir.is_none() {
                task.cache_dir = defaults.cache_dir.clone();
            }
            if task.max_tokens.is_none() {
                task.max_tokens = defaults.max_tokens;
            }
            if task.temperature.is_none() {
                task.temperature = defaults.temperature;
            }
            if task.top_k.is_none() {
                task.top_k = defaults.top_k;
            }
            if task.top_p.is_none() {
                task.top_p = defaults.top_p;
            }
            if task.ctx_size.is_none() {
                task.ctx_size = defaults.ctx_size;
            }
            if task.threads.is_none() {
                task.threads = defaults.threads;
            }
            if defaults.verbose.unwrap_or(false) && !task.verbose {
                task.verbose = true;
            }
            if defaults.no_color.unwrap_or(false) && !task.no_color {
                task.no_color = true;
            }
            if defaults.stats.unwrap_or(false) && !task.stats {
                task.stats = true;
            }
        }
    }
    
    /// Generate a sample configuration file
    pub fn generate_sample() -> Self {
        let mut environment = HashMap::new();
        environment.insert("RUSTLAMA_VERBOSE".to_string(), "true".to_string());
        
        Self {
            version: "1.0".to_string(),
            name: Some("RustLama Configuration".to_string()),
            description: Some("Example configuration for batch inference and model management".to_string()),
            defaults: Some(DefaultConfig {
                model: Some("TheBloke/Llama-2-7B-Chat-GGUF".to_string()),
                hf_filename: None,
                cache_dir: None,
                max_tokens: Some(1024),
                temperature: Some(0.8),
                top_k: Some(40),
                top_p: Some(0.95),
                ctx_size: Some(2048),
                threads: None,
                verbose: Some(false),
                no_color: Some(false),
                stats: Some(false),
            }),
            models: vec![
                ModelTask {
                    action: "pull".to_string(),
                    model_id: Some("TheBloke/Llama-2-7B-Chat-GGUF".to_string()),
                    filename: Some("llama-2-7b-chat.Q4_K_M.gguf".to_string()),
                    cache_dir: None,
                    force: false,
                    verbose: true,
                    description: Some("Download Llama 2 7B Chat model".to_string()),
                }
            ],
            tasks: vec![
                InferenceTask {
                    name: "Creative Writing".to_string(),
                    prompt: "Write a short story about space exploration".to_string(),
                    model: None,
                    hf_filename: None,
                    cache_dir: None,
                    force_download: false,
                    max_tokens: Some(512),
                    temperature: Some(1.0),
                    top_k: Some(40),
                    top_p: Some(0.9),
                    ctx_size: None,
                    threads: None,
                    no_color: false,
                    stats: true,
                    verbose: false,
                    output_file: Some("creative_story.txt".to_string()),
                    description: Some("Generate creative content".to_string()),
                    continue_on_error: false,
                },
                InferenceTask {
                    name: "Technical Explanation".to_string(),
                    prompt: "Explain how neural networks work in simple terms".to_string(),
                    model: None,
                    hf_filename: None,
                    cache_dir: None,
                    force_download: false,
                    max_tokens: Some(1024),
                    temperature: Some(0.3),
                    top_k: Some(20),
                    top_p: Some(0.95),
                    ctx_size: None,
                    threads: None,
                    no_color: false,
                    stats: true,
                    verbose: true,
                    output_file: Some("neural_networks.txt".to_string()),
                    description: Some("Generate technical documentation".to_string()),
                    continue_on_error: false,
                },
            ],
            environment,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_yaml_config_validation() {
        let config = YamlConfig::generate_sample();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_yaml_config_serialization() {
        let config = YamlConfig::generate_sample();
        let yaml_str = serde_yaml::to_string(&config).unwrap();
        let parsed_config: YamlConfig = serde_yaml::from_str(&yaml_str).unwrap();
        assert_eq!(config.version, parsed_config.version);
    }

    #[test]
    fn test_yaml_config_file_io() -> Result<()> {
        let config = YamlConfig::generate_sample();
        
        // Create temporary file
        let mut temp_file = NamedTempFile::new()?;
        let yaml_content = serde_yaml::to_string(&config)?;
        temp_file.write_all(yaml_content.as_bytes())?;
        
        // Load from file
        let loaded_config = YamlConfig::load_from_file(temp_file.path())?;
        assert_eq!(config.version, loaded_config.version);
        
        Ok(())
    }
}
