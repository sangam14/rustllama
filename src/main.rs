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
use clap::{Parser, Subcommand};
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::{AddBos, LlamaModel, Special};
use std::fs;
use std::io::{self, Write};
use std::num::NonZeroU32;
use std::path::PathBuf;
use std::time::Instant;

#[cfg(test)]
mod tests;
mod downloader;
mod config;

use downloader::{is_hf_model_id, ModelDownloader};
use config::{YamlConfig, InferenceTask, ModelTask};

#[derive(Parser)]
#[command(
    name = "rustlama",
    version,
    about = "Fast LLaMA model inference CLI powered by llama.cpp",
    long_about = "A high-performance command-line interface for running inference with LLaMA models.\n\
                  Supports GGUF format models with configurable sampling parameters.\n\n\
                  EXAMPLES:\n\
                    # Generate text with a model\n\
                    rustlama run --model TheBloke/Llama-2-7B-Chat-GGUF --prompt \"Hello world\"\n\n\
                    # List cached models\n\
                    rustlama models ls\n\n\
                    # Download a model\n\
                    rustlama models pull TheBloke/Llama-2-7B-Chat-GGUF --filename llama-2-7b-chat.Q4_K_M.gguf\n\n\
                    # Remove a model\n\
                    rustlama models rm TheBloke/Llama-2-7B-Chat-GGUF\n\n\
                    # Generate sample YAML config\n\
                    rustlama config --generate-sample --output my-tasks.yml\n\n\
                    # Run batch tasks from YAML\n\
                    rustlama config --file my-tasks.yml",
    author = "Sangam Biradar"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run inference with a model (default command)
    Run {
        /// Path to the GGUF model file or Hugging Face model ID
        #[arg(short, long, value_name = "FILE_OR_HF_ID", help = "Path to GGUF model file or Hugging Face model ID")]
        model: String,

        /// Hugging Face model filename (for HF models)
        #[arg(long, help = "Specific filename to download from HF model (auto-detected if not specified)")]
        hf_filename: Option<String>,

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
    },

    /// Manage models (pull, list, remove)
    Models {
        #[command(subcommand)]
        command: ModelCommands,
    },

    /// Run tasks from YAML configuration file
    Config {
        /// Path to YAML configuration file
        #[arg(short, long, help = "Path to YAML configuration file")]
        file: Option<PathBuf>,

        /// Dry run - show what would be executed without running
        #[arg(long, help = "Show what would be executed without actually running")]
        dry_run: bool,

        /// Generate sample configuration file
        #[arg(long, help = "Generate a sample configuration file")]
        generate_sample: bool,

        /// Output file for sample generation
        #[arg(long, default_value = "rustlama.yml", help = "Output file for sample configuration")]
        output: PathBuf,

        /// Continue execution on errors
        #[arg(long, help = "Continue executing remaining tasks even if some fail")]
        continue_on_error: bool,

        /// Only run specific tasks (by name)
        #[arg(long, help = "Only run specific tasks (comma-separated names)")]
        only_tasks: Option<String>,

        /// Skip specific tasks (by name)  
        #[arg(long, help = "Skip specific tasks (comma-separated names)")]
        skip_tasks: Option<String>,

        /// Verbose output
        #[arg(short, long, help = "Enable verbose output")]
        verbose: bool,
    },
}

#[derive(Subcommand)]
enum ModelCommands {
    /// Download/pull a model from Hugging Face
    Pull {
        /// Hugging Face model ID
        #[arg(help = "Hugging Face model ID (e.g., TheBloke/Llama-2-7B-Chat-GGUF)")]
        model_id: String,

        /// Specific filename to download
        #[arg(long, help = "Specific filename to download (auto-detected if not specified)")]
        filename: Option<String>,

        /// Models cache directory
        #[arg(long, help = "Directory to cache downloaded models (default: ~/.cache/rustlama)")]
        cache_dir: Option<String>,

        /// Force re-download even if model exists
        #[arg(short, long, help = "Force re-download model even if it exists locally")]
        force: bool,

        /// Verbose output
        #[arg(short, long, help = "Enable verbose output")]
        verbose: bool,
    },

    /// List cached models
    #[command(alias = "ls")]
    List {
        /// Models cache directory
        #[arg(long, help = "Directory to check for cached models (default: ~/.cache/rustlama)")]
        cache_dir: Option<String>,

        /// Show detailed information
        #[arg(short, long, help = "Show detailed model information")]
        verbose: bool,
    },

    /// Remove a cached model
    #[command(alias = "rm")]
    Remove {
        /// Hugging Face model ID or local path pattern
        #[arg(help = "Model ID to remove (e.g., TheBloke/Llama-2-7B-Chat-GGUF) or 'all' to remove all")]
        model_id: String,

        /// Models cache directory
        #[arg(long, help = "Directory to check for cached models (default: ~/.cache/rustlama)")]
        cache_dir: Option<String>,

        /// Force removal without confirmation
        #[arg(short, long, help = "Force removal without confirmation prompt")]
        force: bool,

        /// Verbose output
        #[arg(short, long, help = "Enable verbose output")]
        verbose: bool,
    },

    /// Show disk usage of cached models
    #[command(alias = "du")]
    Usage {
        /// Models cache directory
        #[arg(long, help = "Directory to check for cached models (default: ~/.cache/rustlama)")]
        cache_dir: Option<String>,
    },
}

fn main() -> Result<()> {
    tokio::runtime::Runtime::new()?.block_on(async_main())
}

async fn async_main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run {
            model,
            hf_filename,
            cache_dir,
            force_download,
            prompt,
            max_tokens,
            temperature,
            top_k,
            top_p,
            ctx_size,
            threads,
            no_color,
            stats,
            verbose,
        } => {
            // Create a compatible structure for the existing inference logic
            let run_config = RunConfig {
                model,
                hf_filename,
                cache_dir,
                force_download,
                prompt,
                max_tokens,
                temperature,
                top_k,
                top_p,
                ctx_size,
                threads,
                no_color,
                stats,
                verbose,
            };
            let _generated_text = run_inference(run_config).await?;
            Ok(())
        }
        Commands::Models { command } => {
            handle_model_commands(command).await
        }
        Commands::Config { 
            file, 
            dry_run, 
            generate_sample, 
            output, 
            continue_on_error, 
            only_tasks, 
            skip_tasks, 
            verbose 
        } => {
            handle_config_command(
                file, 
                dry_run, 
                generate_sample, 
                output, 
                continue_on_error, 
                only_tasks, 
                skip_tasks, 
                verbose
            ).await
        }
    }
}

// Helper struct to maintain compatibility with existing code
pub struct RunConfig {
    model: String,
    hf_filename: Option<String>,
    cache_dir: Option<String>,
    force_download: bool,
    prompt: String,
    max_tokens: usize,
    temperature: f32,
    top_k: usize,
    top_p: f32,
    ctx_size: Option<u32>,
    threads: Option<i32>,
    no_color: bool,
    stats: bool,
    verbose: bool,
}

async fn run_inference(cli: RunConfig) -> Result<String> {
    // Validate inputs
    validate_args(&cli)?;

    if cli.verbose {
        print_banner(&cli);
    }

    // Resolve model path (download if necessary)
    let model_path = if is_hf_model_id(&cli.model) {
        // Download from Hugging Face
        if cli.verbose {
            println!(
                "{} Detected Hugging Face model ID: {}",
                "Info:".blue().bold(),
                cli.model
            );
        }
        
        let downloader = ModelDownloader::new(cli.cache_dir.clone())?;
        
        // If no specific filename provided, try to auto-detect
        let filename_to_download = if let Some(filename) = &cli.hf_filename {
            filename.clone()
        } else {
            // List available files and try to find a suitable one
            if cli.verbose {
                println!("{} Checking available files...", "Info:".blue().bold());
            }
            match downloader.list_model_files(&cli.model).await {
                Ok(files) if !files.is_empty() => {
                    if cli.verbose {
                        println!("{} Available GGUF files:", "Info:".blue().bold());
                        for file in &files {
                            println!("  • {}", file);
                        }
                    }
                    
                    // Try to find a good default (prefer .gguf files)
                    let gguf_files: Vec<_> = files.iter().filter(|f| f.ends_with(".gguf")).collect();
                    if let Some(first_gguf) = gguf_files.first() {
                        if cli.verbose && files.len() > 1 {
                            println!(
                                "{} Auto-selected: {}",
                                "Info:".blue().bold(),
                                first_gguf
                            );
                        }
                        (*first_gguf).clone()
                    } else {
                        files[0].clone()
                    }
                },
                _ => "model.gguf".to_string(), // fallback
            }
        };
        
        downloader.download_model(&cli.model, &filename_to_download, cli.force_download).await?
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
                "{} If this is a Hugging Face model ID, use 'rustlama models pull <model>' first.",
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

    Ok(generated_text)
}

async fn handle_model_commands(command: ModelCommands) -> Result<()> {
    match command {
        ModelCommands::Pull { model_id, filename, cache_dir, force, verbose } => {
            pull_model(model_id, filename, cache_dir, force, verbose).await
        }
        ModelCommands::List { cache_dir, verbose } => {
            list_models(cache_dir, verbose).await
        }
        ModelCommands::Remove { model_id, cache_dir, force, verbose } => {
            remove_models(model_id, cache_dir, force, verbose).await
        }
        ModelCommands::Usage { cache_dir } => {
            show_disk_usage(cache_dir).await
        }
    }
}

async fn pull_model(model_id: String, filename: Option<String>, cache_dir: Option<String>, force: bool, verbose: bool) -> Result<()> {
    if verbose {
        println!("{} Pulling model: {}", "Info:".blue().bold(), model_id.green());
    }

    let downloader = ModelDownloader::new(cache_dir)?;
    
    let filename_to_download = if let Some(filename) = filename {
        filename
    } else {
        if verbose {
            println!("{} No filename specified, detecting available files...", "Info:".blue().bold());
        }
        
        match downloader.list_model_files(&model_id).await {
            Ok(files) => {
                if files.len() == 1 {
                    files[0].clone()
                } else if files.len() > 1 {
                    println!("{} Available files for {}:", "Info:".blue().bold(), model_id.green());
                    for (i, file) in files.iter().enumerate() {
                        println!("  {}. {}", i + 1, file);
                    }
                    
                    // Try to find a reasonable default
                    let gguf_files: Vec<_> = files.iter().filter(|f| f.ends_with(".gguf")).collect();
                    if gguf_files.len() == 1 {
                        let selected = gguf_files[0].clone();
                        println!("{} Auto-selected: {}", "Info:".blue().bold(), selected.green());
                        selected
                    } else {
                        return Err(anyhow::anyhow!(
                            "Multiple files available. Please specify one with --filename:\n{}",
                            files.iter().map(|f| format!("  {}", f)).collect::<Vec<_>>().join("\n")
                        ));
                    }
                } else {
                    return Err(anyhow::anyhow!("No files found for model: {}", model_id));
                }
            }
            Err(e) => return Err(anyhow::anyhow!("Failed to list model files: {}", e)),
        }
    };

    let path = downloader.download_model(&model_id, &filename_to_download, force).await?;
    println!("{} Model pulled successfully: {}", "Success:".green().bold(), path.display());
    Ok(())
}

async fn list_models(cache_dir: Option<String>, verbose: bool) -> Result<()> {
    let downloader = ModelDownloader::new(cache_dir)?;
    let cache_path = downloader.get_cache_dir();
    
    if !cache_path.exists() {
        println!("{} No models cached. Use 'rustlama models pull <model>' to download models.", "Info:".blue().bold());
        return Ok(());
    }

    println!("{} Cached models in: {}", "Models:".green().bold(), cache_path.display());
    println!();

    let mut total_size = 0u64;
    let mut model_count = 0;

    for entry in fs::read_dir(&cache_path)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            let model_dir = entry.path();
            let model_name = model_dir.file_name().unwrap().to_string_lossy();
            
            // Convert back from filesystem safe name
            let display_name = model_name.replace("--", "/");
            
            println!("📦 {}", display_name.cyan().bold());
            
            if verbose {
                for model_file in fs::read_dir(&model_dir)? {
                    let model_file = model_file?;
                    if model_file.file_type()?.is_file() {
                        let metadata = model_file.metadata()?;
                        let size = metadata.len();
                        total_size += size;
                        
                        println!("   └─ {} ({})", 
                            model_file.file_name().to_string_lossy(),
                            format_file_size(size).yellow()
                        );
                    }
                }
            } else {
                // Just count files and sizes without verbose output
                for model_file in fs::read_dir(&model_dir)? {
                    let model_file = model_file?;
                    if model_file.file_type()?.is_file() {
                        let metadata = model_file.metadata()?;
                        total_size += metadata.len();
                    }
                }
            }
            model_count += 1;
        }
    }

    println!();
    println!("{} {} models, {} total", 
        "Summary:".green().bold(),
        model_count, 
        format_file_size(total_size).yellow()
    );
    
    if !verbose && model_count > 0 {
        println!("{} Use --verbose for detailed information", "Tip:".blue().bold());
    }

    Ok(())
}

async fn remove_models(model_id: String, cache_dir: Option<String>, force: bool, verbose: bool) -> Result<()> {
    let downloader = ModelDownloader::new(cache_dir)?;
    let cache_path = downloader.get_cache_dir();
    
    if !cache_path.exists() {
        println!("{} No cached models found.", "Info:".blue().bold());
        return Ok(());
    }

    if model_id == "all" {
        return remove_all_models(cache_path.clone(), force, verbose).await;
    }

    // Convert model ID to filesystem safe name
    let safe_model_name = model_id.replace("/", "--");
    let model_path = cache_path.join(&safe_model_name);

    if !model_path.exists() {
        println!("{} Model '{}' not found in cache.", "Error:".red().bold(), model_id);
        return Ok(());
    }

    if !force {
        print!("Remove model '{}'? [y/N]: ", model_id.yellow());
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        if !input.trim().to_lowercase().starts_with('y') {
            println!("Cancelled.");
            return Ok(());
        }
    }

    if verbose {
        println!("{} Removing model: {}", "Info:".blue().bold(), model_id.yellow());
    }

    fs::remove_dir_all(&model_path)?;
    println!("{} Model '{}' removed successfully.", "Success:".green().bold(), model_id);

    Ok(())
}

async fn remove_all_models(cache_path: PathBuf, force: bool, verbose: bool) -> Result<()> {
    if !force {
        print!("Remove ALL cached models? This cannot be undone! [y/N]: ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        if !input.trim().to_lowercase().starts_with('y') {
            println!("Cancelled.");
            return Ok(());
        }
    }

    if verbose {
        println!("{} Removing all cached models...", "Info:".blue().bold());
    }

    fs::remove_dir_all(&cache_path)?;
    fs::create_dir_all(&cache_path)?;
    
    println!("{} All models removed successfully.", "Success:".green().bold());
    Ok(())
}

async fn show_disk_usage(cache_dir: Option<String>) -> Result<()> {
    let downloader = ModelDownloader::new(cache_dir)?;
    let cache_path = downloader.get_cache_dir();
    
    if !cache_path.exists() {
        println!("{} No cached models found.", "Info:".blue().bold());
        return Ok(());
    }

    println!("{} Disk usage for: {}", "Usage:".green().bold(), cache_path.display());
    println!();

    let mut total_size = 0u64;
    let mut models = Vec::new();

    for entry in fs::read_dir(&cache_path)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            let model_dir = entry.path();
            let model_name = model_dir.file_name().unwrap().to_string_lossy();
            let display_name = model_name.replace("--", "/");
            
            let mut model_size = 0u64;
            for model_file in fs::read_dir(&model_dir)? {
                let model_file = model_file?;
                if model_file.file_type()?.is_file() {
                    let metadata = model_file.metadata()?;
                    model_size += metadata.len();
                }
            }
            
            total_size += model_size;
            models.push((display_name.to_string(), model_size));
        }
    }

    // Sort by size (largest first)
    models.sort_by(|a, b| b.1.cmp(&a.1));

    for (name, size) in models {
        println!("{:>12} {}", format_file_size(size).yellow(), name.cyan());
    }

    println!("{}", "─".repeat(50));
    println!("{:>12} {}", format_file_size(total_size).green().bold(), "Total");

    Ok(())
}

fn format_file_size(size: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = size as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if size >= 10.0 {
        format!("{:.1}{}", size, UNITS[unit_index])
    } else {
        format!("{:.2}{}", size, UNITS[unit_index])
    }
}

async fn handle_config_command(
    file: Option<PathBuf>,
    dry_run: bool,
    generate_sample: bool,
    output: PathBuf,
    continue_on_error: bool,
    only_tasks: Option<String>,
    skip_tasks: Option<String>,
    verbose: bool,
) -> Result<()> {
    // Generate sample configuration if requested
    if generate_sample {
        let sample_config = YamlConfig::generate_sample();
        sample_config.save_to_file(&output)?;
        println!("{} Sample configuration generated: {}", 
                 "Success:".green().bold(), 
                 output.display());
        return Ok(());
    }

    // Require file for non-sample operations
    let config_file = file.ok_or_else(|| {
        anyhow::anyhow!("Configuration file is required unless --generate-sample is used")
    })?;

    // Load configuration from file
    if verbose {
        println!("{} Loading configuration from: {}", 
                 "Info:".blue().bold(), 
                 config_file.display());
    }

    let config = YamlConfig::load_from_file(&config_file)?;

    if verbose {
        if let Some(name) = &config.name {
            println!("{} Configuration: {}", "Info:".blue().bold(), name);
        }
        if let Some(description) = &config.description {
            println!("{} {}", "Description:".blue().bold(), description);
        }
    }

    // Parse task filters
    let only_task_names: Option<Vec<String>> = only_tasks.map(|tasks| {
        tasks.split(',').map(|s| s.trim().to_string()).collect()
    });

    let skip_task_names: Option<Vec<String>> = skip_tasks.map(|tasks| {
        tasks.split(',').map(|s| s.trim().to_string()).collect()
    });

    // Execute model management tasks
    if !config.models.is_empty() {
        println!("{} Executing model management tasks...", "Info:".blue().bold());
        
        for (i, model_task) in config.models.iter().enumerate() {
            if let Some(desc) = &model_task.description {
                if verbose {
                    println!("{} Model Task {}: {}", "Info:".blue().bold(), i + 1, desc);
                }
            }

            if dry_run {
                println!("  {} Would execute: {} {:?}", 
                         "DRY RUN:".yellow().bold(),
                         model_task.action,
                         model_task.model_id.as_deref().unwrap_or("N/A"));
                continue;
            }

            if let Err(e) = execute_model_task(model_task).await {
                eprintln!("{} Model task {} failed: {}", 
                          "Error:".red().bold(), i + 1, e);
                if !continue_on_error {
                    return Err(e);
                }
            }
        }
    }

    // Execute inference tasks - clone tasks to avoid borrow issues
    let tasks = config.tasks.clone();
    if !tasks.is_empty() {
        println!("{} Executing inference tasks...", "Info:".blue().bold());
        
        let mut executed_count = 0;
        let mut failed_count = 0;

        for mut task in tasks {
            // Apply default settings
            config.apply_defaults(&mut task);

            // Check task filters
            if let Some(ref only_names) = only_task_names {
                if !only_names.contains(&task.name) {
                    continue;
                }
            }

            if let Some(ref skip_names) = skip_task_names {
                if skip_names.contains(&task.name) {
                    if verbose {
                        println!("{} Skipping task: {}", "Info:".blue().bold(), task.name);
                    }
                    continue;
                }
            }

            if verbose {
                println!("{} Executing task: {}", "Info:".blue().bold(), task.name);
                if let Some(desc) = &task.description {
                    println!("  {}", desc);
                }
            }

            if dry_run {
                println!("  {} Would run: {} with model {:?}", 
                         "DRY RUN:".yellow().bold(),
                         task.name,
                         task.model.as_deref().unwrap_or("default"));
                continue;
            }

            match execute_inference_task(&task, verbose).await {
                Ok(()) => {
                    executed_count += 1;
                    println!("{} Task '{}' completed successfully", 
                             "Success:".green().bold(), task.name);
                }
                Err(e) => {
                    failed_count += 1;
                    eprintln!("{} Task '{}' failed: {}", 
                              "Error:".red().bold(), task.name, e);
                    if !continue_on_error {
                        return Err(e);
                    }
                }
            }
        }

        if !dry_run {
            println!("\n{} Batch execution complete!", "Summary:".green().bold());
            println!("  • {} tasks executed successfully", executed_count);
            if failed_count > 0 {
                println!("  • {} tasks failed", failed_count);
            }
        }
    }

    Ok(())
}

async fn execute_model_task(task: &ModelTask) -> Result<()> {
    match task.action.as_str() {
        "pull" => {
            let model_id = task.model_id.as_ref()
                .ok_or_else(|| anyhow::anyhow!("Model ID is required for pull action"))?;
            pull_model(
                model_id.clone(),
                task.filename.clone(),
                task.cache_dir.clone(),
                task.force,
                task.verbose,
            ).await
        }
        "remove" => {
            let model_id = task.model_id.as_ref()
                .ok_or_else(|| anyhow::anyhow!("Model ID is required for remove action"))?;
            remove_models(
                model_id.clone(),
                task.cache_dir.clone(),
                task.force,
                task.verbose,
            ).await
        }
        "list" => {
            list_models(task.cache_dir.clone(), task.verbose).await
        }
        "usage" => {
            show_disk_usage(task.cache_dir.clone()).await
        }
        _ => Err(anyhow::anyhow!("Unknown model action: {}", task.action))
    }
}

async fn execute_inference_task(task: &InferenceTask, global_verbose: bool) -> Result<()> {
    let model = task.model.as_ref()
        .ok_or_else(|| anyhow::anyhow!("Model is required for inference task '{}'", task.name))?;

    // Create RunConfig from the task
    let run_config = RunConfig {
        model: model.clone(),
        hf_filename: task.hf_filename.clone(),
        cache_dir: task.cache_dir.clone(),
        force_download: task.force_download,
        prompt: task.prompt.clone(),
        max_tokens: task.max_tokens.unwrap_or(1024),
        temperature: task.temperature.unwrap_or(0.8),
        top_k: task.top_k.unwrap_or(40),
        top_p: task.top_p.unwrap_or(0.95),
        ctx_size: task.ctx_size,
        threads: task.threads,
        no_color: task.no_color,
        stats: task.stats,
        verbose: task.verbose || global_verbose,
    };

    // Capture output if output_file is specified
    if let Some(output_file) = &task.output_file {
        let generated_text = run_inference(run_config).await?;
        
        // Save the generated text to file
        match fs::write(output_file, &generated_text) {
            Ok(()) => {
                if global_verbose {
                    println!("  {} Output saved to: {}", 
                             "Success:".green().bold(), output_file);
                }
            }
            Err(e) => {
                eprintln!("  {} Failed to save output to {}: {}", 
                         "Error:".red().bold(), output_file, e);
                return Err(anyhow::anyhow!("Failed to save output to file: {}", e));
            }
        }
        
        Ok(())
    } else {
        let _generated_text = run_inference(run_config).await?;
        Ok(())
    }
}

pub fn validate_args(cli: &RunConfig) -> Result<()> {
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

fn print_banner(cli: &RunConfig) {
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

fn print_stats(tokens_generated: usize, duration: std::time::Duration, cli: &RunConfig) {
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
