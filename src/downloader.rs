use anyhow::{anyhow, Result};
use colored::*;
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

/// Hugging Face model information response
#[derive(Debug, Deserialize, Serialize)]
pub struct HfModelInfo {
    pub id: String,
    pub siblings: Vec<HfFile>,
}

/// Hugging Face file information
#[derive(Debug, Deserialize, Serialize)]
pub struct HfFile {
    pub rfilename: String,
    #[serde(rename = "size")]
    pub size: Option<u64>,
}

/// Model downloader for Hugging Face models
pub struct ModelDownloader {
    client: reqwest::Client,
    cache_dir: PathBuf,
}

impl ModelDownloader {
    /// Create a new model downloader
    pub fn new(cache_dir: Option<String>) -> Result<Self> {
        let cache_dir = if let Some(dir) = cache_dir {
            PathBuf::from(dir)
        } else {
            // Default cache directory: ~/.cache/rustlama
            let home = dirs::home_dir()
                .ok_or_else(|| anyhow!("Could not determine home directory"))?;
            home.join(".cache").join("rustlama")
        };

        // Create cache directory if it doesn't exist
        fs::create_dir_all(&cache_dir)
            .map_err(|e| anyhow!("Failed to create cache directory: {}", e))?;

        let client = reqwest::Client::builder()
            .user_agent("rustlama/0.1.0")
            .build()
            .map_err(|e| anyhow!("Failed to create HTTP client: {}", e))?;

        Ok(Self { client, cache_dir })
    }

    /// Get the local path for a model
    pub fn get_model_path(&self, model_id: &str, filename: &str) -> PathBuf {
        let safe_model_id = model_id.replace('/', "--");
        self.cache_dir
            .join("models")
            .join(safe_model_id)
            .join(filename)
    }

    /// Check if a model file exists locally
    pub fn model_exists(&self, model_id: &str, filename: &str) -> bool {
        self.get_model_path(model_id, filename).exists()
    }

    /// Get model information from Hugging Face Hub
    pub async fn get_model_info(&self, model_id: &str) -> Result<HfModelInfo> {
        let url = format!("https://huggingface.co/api/models/{}", model_id);
        
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to fetch model info: {}", e))?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Failed to fetch model info: HTTP {}",
                response.status()
            ));
        }

        let model_info: HfModelInfo = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse model info: {}", e))?;

        Ok(model_info)
    }

    /// Download a model file from Hugging Face Hub
    pub async fn download_model(
        &self,
        model_id: &str,
        filename: &str,
        force_download: bool,
    ) -> Result<PathBuf> {
        let local_path = self.get_model_path(model_id, filename);

        // Check if file already exists
        if local_path.exists() && !force_download {
            println!(
                "{} Model already exists: {}",
                "Info:".blue().bold(),
                local_path.display()
            );
            return Ok(local_path);
        }

        // Create parent directories
        if let Some(parent) = local_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| anyhow!("Failed to create model directory: {}", e))?;
        }

        println!(
            "{} Downloading model: {} (file: {})",
            "Info:".blue().bold(),
            model_id,
            filename
        );

        // Get model info to find the file
        let model_info = self.get_model_info(model_id).await?;
        
        let file_info = model_info
            .siblings
            .iter()
            .find(|f| f.rfilename == filename)
            .ok_or_else(|| anyhow!("File '{}' not found in model '{}'", filename, model_id))?;

        let file_size = file_info.size.unwrap_or(0);

        // Download URL
        let download_url = format!(
            "https://huggingface.co/{}/resolve/main/{}",
            model_id, filename
        );

        // Start download
        let response = self
            .client
            .get(&download_url)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to start download: {}", e))?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Failed to download file: HTTP {}",
                response.status()
            ));
        }

        // Create progress bar
        let pb = ProgressBar::new(file_size);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")
                .unwrap()
                .progress_chars("#>-"),
        );

        // Create temporary file
        let temp_path = local_path.with_extension("tmp");
        let mut file = File::create(&temp_path)
            .map_err(|e| anyhow!("Failed to create temporary file: {}", e))?;

        let mut downloaded = 0u64;
        let mut stream = response.bytes_stream();
        let mut hasher = Sha256::new();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| anyhow!("Failed to read chunk: {}", e))?;
            
            file.write_all(&chunk)
                .map_err(|e| anyhow!("Failed to write chunk: {}", e))?;
            
            hasher.update(&chunk);
            downloaded += chunk.len() as u64;
            pb.set_position(downloaded);
        }

        pb.finish_with_message("Download complete!".green().to_string());

        // Close file and rename from temp
        drop(file);
        fs::rename(&temp_path, &local_path)
            .map_err(|e| anyhow!("Failed to finalize download: {}", e))?;

        println!(
            "{} Model downloaded successfully: {}",
            "Success:".green().bold(),
            local_path.display()
        );

        Ok(local_path)
    }

    /// List available files for a model
    pub async fn list_model_files(&self, model_id: &str) -> Result<Vec<String>> {
        let model_info = self.get_model_info(model_id).await?;
        
        let gguf_files: Vec<String> = model_info
            .siblings
            .into_iter()
            .filter(|f| f.rfilename.ends_with(".gguf"))
            .map(|f| f.rfilename)
            .collect();

        Ok(gguf_files)
    }

    /// Get the cache directory path
    pub fn get_cache_dir(&self) -> &PathBuf {
        &self.cache_dir
    }
}

/// Check if a string looks like a Hugging Face model ID
pub fn is_hf_model_id(model: &str) -> bool {
    // HF model IDs are typically in the format "username/modelname" or "organization/modelname"
    // and don't contain file extensions or paths
    if model.contains('/') 
        && !model.starts_with('/') 
        && !model.starts_with('.') 
        && !model.ends_with(".gguf") 
        && !model.contains('\\') 
        && !Path::new(model).exists() {
        // Count slashes - should be exactly one for typical HF model IDs
        model.matches('/').count() == 1
    } else {
        false
    }
}
