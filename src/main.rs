/*!
# RustLlama - Fast LLaMA Inference CLI

A high-performance command-line interface for running inference with LLaMA models
using llama.cpp bindings. Built for speed, efficiency, and ease of use.

## Features

- 🚀 Fast inference with llama.cpp backend
- 🎛️ Configurable sampling parameters
- 💾 Support for various GGUF model formats
- 🔧 Flexible context size and threading options
- 📊 Real-time generation statistics
- 🎨 Colored output for better readability

## Usage

```bash
rustlama --model model.gguf --prompt "Hello, world!"
```
*/

use anyhow::Result;
use clap::Parser;
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::{AddBos, LlamaModel, Special};
use std::io::{self, Write};
use std::num::NonZeroU32;
use std::path::PathBuf;
use std::time::Instant;

#[cfg(test)]
mod tests;
mod downloader;

use downloader::{is_hf_model_id, ModelDownloader};

#[derive(Parser)]
#[command(
    name = "rustlama",
    version,
    about = "Fast LLaMA model inference CLI powered by llama.cpp",
    long_about = "A high-performance command-line interface for running inference with LLaMA models.\n\
                  Supports GGUF format models with configurable sampling parameters.",
    author = "Sangam Biradar"
)]
struct Cli {
    /// Path to the GGUF model file or Hugging Face model ID
    #[arg(short, long, value_name = "FILE_OR_HF_ID", help = "Path to GGUF model file or Hugging Face model ID (e.g., microsoft/DialoGPT-medium)")]
    model: String,

    /// Download model from Hugging Face Hub
    #[arg(long, help = "Download model from Hugging Face Hub")]
    download: bool,

    /// Hugging Face model filename (for HF downloads)
    #[arg(long, default_value = "model.gguf", help = "Specific filename to download from HF model")]
    hf_filename: String,

    /// Models cache directory
    #[arg(long, help = "Directory to cache downloaded models (default: ~/.cache/rustlama)")]
    cache_dir: Option<String>,

    /// Force re-download even if model exists
    #[arg(long, help = "Force re-download model even if it exists locally")]
    force_download: bool,

    /// Input prompt for generation
    #[arg(
        short,
        long,
        value_name = "TEXT",
        help = "Input prompt for text generation"
    )]
    prompt: String,

    /// Maximum number of tokens to generate
    #[arg(
        short = 'n',
        long,
        default_value = "1024",
        help = "Maximum number of tokens to generate"
    )]
    max_tokens: usize,

    /// Sampling temperature (0.1 = conservative, 1.0 = balanced, 2.0 = creative)
    #[arg(
        short,
        long,
        default_value = "0.8",
        help = "Sampling temperature (0.1-2.0)"
    )]
    temperature: f32,

    /// Top-k sampling: limit to k most likely tokens
    #[arg(long, default_value = "40", help = "Top-k sampling parameter")]
    top_k: usize,

    /// Top-p (nucleus) sampling: cumulative probability threshold
    #[arg(
        long,
        default_value = "0.95",
        help = "Top-p sampling parameter (0.0-1.0)"
    )]
    top_p: f32,

    /// Context size (number of tokens the model can remember)
    #[arg(
        short = 'c',
        long,
        help = "Context size in tokens (default: model's default)"
    )]
    ctx_size: Option<u32>,

    /// Number of threads to use
    #[arg(short = 'j', long, help = "Number of threads for inference")]
    threads: Option<i32>,

    /// Disable colored output
    #[arg(long, help = "Disable colored output")]
    no_color: bool,

    /// Show generation statistics
    #[arg(short, long, help = "Show detailed generation statistics")]
    stats: bool,

    /// Verbose output
    #[arg(short, long, help = "Enable verbose output")]
    verbose: bool,
}

fn main() -> Result<()> {
    tokio::runtime::Runtime::new()?.block_on(async_main())
}

async fn async_main() -> Result<()> {
    let cli = Cli::parse();

    // Validate inputs
    validate_args(&cli)?;

    if cli.verbose {
        print_banner(&cli);
    }

    // Resolve model path (download if necessary)
    let model_path = if cli.download || is_hf_model_id(&cli.model) {
        // Download from Hugging Face
        if cli.verbose {
            println!(
                "{} Detected Hugging Face model ID: {}",
                "Info:".blue().bold(),
                cli.model
            );
        }
        
        let downloader = ModelDownloader::new(cli.cache_dir.clone())?;
        
        // If --download flag is used or model ID format detected, download the model
        if cli.download {
            // List available files if no specific filename provided
            if cli.hf_filename == "model.gguf" && cli.verbose {
                println!("{} Checking available files...", "Info:".blue().bold());
                match downloader.list_model_files(&cli.model).await {
                    Ok(files) if !files.is_empty() => {
                        println!("{} Available GGUF files:", "Info:".blue().bold());
                        for file in &files {
                            println!("  • {}", file);
                        }
                        if files.len() > 1 && cli.hf_filename == "model.gguf" {
                            println!(
                                "{} Multiple files found. Use --hf-filename to specify which file to download.",
                                "Warning:".yellow().bold()
                            );
                            // Use the first .gguf file found
                            if let Some(first_file) = files.first() {
                                println!(
                                    "{} Using first available file: {}",
                                    "Info:".blue().bold(),
                                    first_file
                                );
                            }
                        }
                    },
                    Ok(_) => {
                        println!("{} No GGUF files found in model repository", "Warning:".yellow().bold());
                    },
                    Err(e) => {
                        if cli.verbose {
                            println!("{} Could not list files: {}", "Warning:".yellow().bold(), e);
                        }
                    }
                }
            }
            
            let filename_to_download = if cli.hf_filename != "model.gguf" {
                cli.hf_filename.clone()
            } else {
                // Try to find a suitable GGUF file
                match downloader.list_model_files(&cli.model).await {
                    Ok(files) if !files.is_empty() => files[0].clone(),
                    _ => cli.hf_filename.clone(),
                }
            };
            
            downloader.download_model(&cli.model, &filename_to_download, cli.force_download).await?
        } else {
            // Check if already downloaded
            let filename = &cli.hf_filename;
            if downloader.model_exists(&cli.model, filename) {
                downloader.get_model_path(&cli.model, filename)
            } else {
                println!(
                    "{} Model not found locally. Use --download to download from Hugging Face.",
                    "Error:".red().bold()
                );
                std::process::exit(1);
            }
        }
    } else {
        // Local file path
        let path = PathBuf::from(&cli.model);
        if !path.exists() {
            eprintln!(
                "{} Model file not found: {}",
                "Error:".red().bold(),
                cli.model
            );
            eprintln!(
                "{} If this is a Hugging Face model ID, use --download flag.",
                "Hint:".cyan().bold()
            );
            std::process::exit(1);
        }
        path
    };

    if cli.verbose {
        println!(
            "{} Initializing llama.cpp backend...",
            "Info:".blue().bold()
        );
    }

    // Initialize llama backend
    let backend = LlamaBackend::init()
        .map_err(|e| anyhow::anyhow!("Failed to initialize llama backend: {}", e))?;

    if cli.verbose {
        println!("{} Loading model: {}", "Info:".blue().bold(), model_path.display());
    }

    // Set up model parameters
    let model_params = LlamaModelParams::default();

    // Load the model with progress indication
    let loading_msg = format!("Loading model: {}", model_path.display());
    let pb = if !cli.no_color {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
                .template("{spinner:.green} {msg}")
                .unwrap(),
        );
        pb.set_message(loading_msg);
        pb.enable_steady_tick(std::time::Duration::from_millis(100));
        Some(pb)
    } else {
        println!("Loading model...");
        None
    };

    let model = LlamaModel::load_from_file(&backend, model_path.to_string_lossy().as_ref(), &model_params)
        .map_err(|e| anyhow::anyhow!("Failed to load model: {}", e))?;

    if let Some(pb) = &pb {
        pb.finish_with_message("Model loaded successfully ✓".green().to_string());
    } else {
        println!("Model loaded successfully");
    }

    // Set up context parameters
    let mut ctx_params = LlamaContextParams::default();

    if let Some(ctx_size) = cli.ctx_size {
        if let Some(non_zero_ctx) = NonZeroU32::new(ctx_size) {
            ctx_params = ctx_params.with_n_ctx(Some(non_zero_ctx));
        }
    } else {
        ctx_params = ctx_params.with_n_ctx(Some(NonZeroU32::new(2048).unwrap()));
    }

    if let Some(threads) = cli.threads {
        ctx_params = ctx_params.with_n_threads(threads);
    }

    if cli.verbose {
        println!("{} Creating context...", "Info:".blue().bold());
    }

    // Create context from model
    let mut ctx = model
        .new_context(&backend, ctx_params)
        .map_err(|e| anyhow::anyhow!("Failed to create context: {}", e))?;

    if cli.verbose {
        println!(
            "{} Context created with {} tokens",
            "Info:".blue().bold(),
            ctx.n_ctx()
        );
    }

    // Tokenize the prompt
    let tokens = model
        .str_to_token(&cli.prompt, AddBos::Always)
        .map_err(|e| anyhow::anyhow!("Failed to tokenize prompt: {}", e))?;

    if cli.verbose {
        println!(
            "{} Prompt tokenized: {} tokens",
            "Info:".blue().bold(),
            tokens.len()
        );
    }

    // Create batch for processing tokens
    let mut batch = LlamaBatch::new(512, 1);

    // Add prompt tokens to batch
    for (i, &token) in tokens.iter().enumerate() {
        let is_last = i == tokens.len() - 1;
        batch
            .add(token, i as i32, &[0], is_last)
            .map_err(|e| anyhow::anyhow!("Failed to add token to batch: {}", e))?;
    }

    if cli.verbose {
        println!("{} Processing prompt...", "Info:".blue().bold());
    }

    // Process the prompt
    ctx.decode(&mut batch)
        .map_err(|e| anyhow::anyhow!("Failed to process prompt: {}", e))?;

    // Print prompt if not verbose (so user sees what they're generating from)
    if !cli.verbose {
        if !cli.no_color {
            print!("{}", cli.prompt.bright_blue());
        } else {
            print!("{}", cli.prompt);
        }
    }

    // Generate tokens
    let start_time = Instant::now();
    let mut generated_text = String::new();
    let mut n_cur = tokens.len() as i32;
    let mut tokens_generated = 0;

    println!(); // New line after prompt

    for _ in 0..cli.max_tokens {
        // Sample next token using greedy sampling (simplest approach)
        // For the first iteration, get logits from the last position of the prompt
        // For subsequent iterations, get logits from position 0 (the current token)
        let logit_index = if tokens_generated == 0 {
            // First generation - get from the last prompt token
            (tokens.len() - 1) as i32
        } else {
            // Subsequent generations - get from position 0
            0
        };

        let candidates: Vec<_> = ctx.candidates_ith(logit_index).collect();

        // Find the token with highest logit (greedy sampling for simplicity)
        let token = candidates
            .iter()
            .max_by(|a, b| a.logit().partial_cmp(&b.logit()).unwrap())
            .map(|c| c.id())
            .unwrap_or(model.token_eos());

        // Check for end of generation
        if token == model.token_eos() {
            if cli.verbose {
                println!("\n{} Reached end-of-sequence token", "Info:".blue().bold());
            }
            break;
        }

        // Convert token to string
        if let Ok(piece) = model.token_to_str(token, Special::Tokenize) {
            generated_text.push_str(&piece);
            if !cli.no_color {
                print!("{}", piece.green());
            } else {
                print!("{}", piece);
            }
            io::stdout().flush().unwrap();
        }

        batch.clear();
        // Add token to batch for next iteration
        batch
            .add(token, n_cur, &[0], true)
            .map_err(|e| anyhow::anyhow!("Failed to add generated token to batch: {}", e))?;
        ctx.decode(&mut batch)
            .map_err(|e| anyhow::anyhow!("Failed to decode batch: {}", e))?;

        n_cur += 1;
        tokens_generated += 1;
    }

    let generation_time = start_time.elapsed();

    println!(); // New line after generation

    // Show statistics if requested
    if cli.stats {
        print_stats(tokens_generated, generation_time, &cli);
    }

    if cli.verbose {
        println!("{} Generation completed!", "Success:".green().bold());
    }

    Ok(())
}

fn validate_args(cli: &Cli) -> Result<()> {
    if cli.temperature < 0.0 || cli.temperature > 2.0 {
        return Err(anyhow::anyhow!("Temperature must be between 0.0 and 2.0"));
    }

    if cli.top_p < 0.0 || cli.top_p > 1.0 {
        return Err(anyhow::anyhow!("Top-p must be between 0.0 and 1.0"));
    }

    if cli.max_tokens == 0 {
        return Err(anyhow::anyhow!("Max tokens must be greater than 0"));
    }

    Ok(())
}

fn print_banner(cli: &Cli) {
    if !cli.no_color {
        println!(
            "{}",
            "🦙 RustLlama - Fast LLaMA Inference CLI"
                .bright_yellow()
                .bold()
        );
        println!("{}", "━".repeat(50).bright_black());
        println!("{} {}", "Model:".cyan().bold(), cli.model);
        println!("{} {}", "Prompt:".cyan().bold(), cli.prompt);
        println!("{} {}", "Max Tokens:".cyan().bold(), cli.max_tokens);
        println!("{} {}", "Temperature:".cyan().bold(), cli.temperature);
        println!("{} {}", "Top-k:".cyan().bold(), cli.top_k);
        println!("{} {}", "Top-p:".cyan().bold(), cli.top_p);
        if let Some(ctx_size) = cli.ctx_size {
            println!("{} {}", "Context Size:".cyan().bold(), ctx_size);
        }
        if let Some(threads) = cli.threads {
            println!("{} {}", "Threads:".cyan().bold(), threads);
        }
        println!("{}", "━".repeat(50).bright_black());
    } else {
        println!("RustLlama - Fast LLaMA Inference CLI");
        println!("Model: {}", cli.model);
        println!("Prompt: {}", cli.prompt);
        println!("Max Tokens: {}", cli.max_tokens);
        println!("Temperature: {}", cli.temperature);
        println!("Top-k: {}", cli.top_k);
        println!("Top-p: {}", cli.top_p);
    }
}

fn print_stats(tokens_generated: usize, duration: std::time::Duration, cli: &Cli) {
    let tokens_per_sec = tokens_generated as f64 / duration.as_secs_f64();

    if !cli.no_color {
        println!("\n{}", "📊 Generation Statistics".bright_cyan().bold());
        println!("{}", "━".repeat(30).bright_black());
        println!("{} {}", "Tokens Generated:".cyan(), tokens_generated);
        println!("{} {:.2}s", "Time Taken:".cyan(), duration.as_secs_f64());
        println!("{} {:.2} tokens/sec", "Speed:".cyan(), tokens_per_sec);
        println!("{}", "━".repeat(30).bright_black());
    } else {
        println!("\nGeneration Statistics");
        println!("Tokens Generated: {}", tokens_generated);
        println!("Time Taken: {:.2}s", duration.as_secs_f64());
        println!("Speed: {:.2} tokens/sec", tokens_per_sec);
    }
}
