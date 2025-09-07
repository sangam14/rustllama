# 🦙 RustLama

[![Rust](https://img.shields.io/badge/rust-stable-brightgreen.svg)](https://rustup.rs/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)

A powerful, Docker-like command-line interface for running LLaMA model inference with advanced workflow automation. Built with Rust for maximum performance, safety, and ease of use.

## ✨ Features

- � **Docker-like CLI**: Intuitive subcommands (`run`, `models pull/ls/rm/du`) for streamlined model management
- �🚀 **High Performance**: Built on llama.cpp with Metal GPU acceleration on Apple Silicon
- 🤗 **Hugging Face Integration**: Seamless model downloading and intelligent caching
- ⚙️ **YAML Workflows**: Declarative batch processing with complex automation pipelines
- 📁 **File Output**: Automatic saving of generated text to files for later review
- 🎛️ **Advanced Sampling**: Temperature, top-k, top-p sampling with stable generation
- 💾 **GGUF Support**: Works with all GGUF format models with auto-quantization detection
- 📝 **Long-form Generation**: Generate up to 1024+ tokens with configurable limits
- 🔧 **Flexible Configuration**: Customizable context size, threading, and cache management
- 📊 **Real-time Statistics**: Generation performance metrics and verbose logging
- 🎨 **Beautiful Output**: Colored terminal output with progress indicators
- 🛡️ **Memory Safe**: Written in Rust with comprehensive error handling
- 🔄 **Batch Processing**: Execute multiple tasks with selective execution and dry-run modes

## 📦 Installation

### From Source

```bash
git clone https://github.com/sangam14/rustlama.git
cd rustlama
cargo build --release
```

The binary will be available at `./target/release/rustlama`

### Using Cargo

```bash
cargo install --git https://github.com/sangam14/rustlama.git
```

## 🚀 Quick Start

### Basic Usage

```bash
# Run inference with automatic file output
rustlama run --model TheBloke/Llama-2-7B-Chat-GGUF --prompt "Write a story about AI"

# Download a specific model file first
rustlama models pull TheBloke/Llama-2-7B-Chat-GGUF --filename "llama-2-7b-chat.Q4_K_M.gguf"

# Then run inference with custom parameters
rustlama run --model TheBloke/Llama-2-7B-Chat-GGUF \
  --prompt "Write a comprehensive guide about machine learning" \
  --max-tokens 1024 \
  --temperature 0.8 \
  --verbose

# Use local model file
rustlama run --model ./path/to/model.gguf --prompt "Hello world"
```

### Model Management (Docker-like commands)

```bash
# List cached models
rustlama models ls

# List with detailed information
rustlama models ls --verbose

# Pull/download a model
rustlama models pull TheBloke/Llama-2-7B-Chat-GGUF --filename "llama-2-7b-chat.Q4_K_M.gguf"

# Remove a specific model
rustlama models rm TheBloke/Llama-2-7B-Chat-GGUF

# Remove all cached models
rustlama models rm all --force

# Show disk usage
rustlama models du
```

### YAML Workflow Automation 🎯

RustLama's most powerful feature is YAML-based workflow automation with automatic file output saving:

```bash
# Generate a sample configuration file
rustlama config --generate-sample --output workflow.yml

# Run automated workflows with file output
rustlama config --file workflow.yml --verbose

# Preview what would be executed (dry run)
rustlama config --file workflow.yml --dry-run

# Run only specific tasks
rustlama config --file workflow.yml --only-tasks "Creative Story,Code Generation"

# Skip specific tasks
rustlama config --file workflow.yml --skip-tasks "heavy-task"

# Continue on errors for batch processing
rustlama config --file workflow.yml --continue-on-error
```

#### Advanced YAML Configuration with File Output

```yaml
# advanced-workflow.yml
version: "1.0"
name: "AI Content Generation Pipeline"
description: "Automated content generation with file output"

# Default settings for all tasks
defaults:
  model: "TheBloke/Llama-2-7B-Chat-GGUF"
  max_tokens: 800
  temperature: 0.8
  verbose: true

# Model management (optional)
models:
  - action: "pull"
    model_id: "TheBloke/Llama-2-7B-Chat-GGUF"
    filename: "llama-2-7b-chat.Q4_K_M.gguf"
    description: "Download main chat model"

# Inference tasks with automatic file output
tasks:
  - name: "Creative Story"
    prompt: "Write an engaging story about space exploration with AI companions"
    max_tokens: 1200
    temperature: 1.1
    output_file: "creative_story.txt"
    description: "Generate creative fiction content"
  
  - name: "Code Generation"
    prompt: "Write Python functions for text preprocessing and sentiment analysis"
    temperature: 0.2
    output_file: "code_generation.txt" 
    description: "Generate clean, documented code"
  
  - name: "Research Summary"
    prompt: "Summarize the current state of large language models in 2024"
    model: "TheBloke/Llama-2-13B-Chat-GGUF"
    max_tokens: 800
    temperature: 0.3
    output_file: "research_summary.txt"
    description: "Generate academic research summary"
```

**Generated Output Files:**
- All generated text is automatically saved to specified files
- Files are created in the current working directory
- Perfect for batch content generation and review workflows
- Supports any file extension (`.txt`, `.md`, `.py`, etc.)

### Advanced Generation Options

```bash
# Generate comprehensive guides with long-form responses  
rustlama run \
  --model TheBloke/Llama-2-7B-Chat-GGUF \
  --hf-filename "llama-2-7b-chat.Q4_K_M.gguf" \
  --prompt "Write a comprehensive guide about machine learning fundamentals" \
  --max-tokens 1024 \
  --verbose

# Creative writing with custom parameters
rustlama run \
  --model TheBloke/Llama-2-7B-Chat-GGUF \
  --prompt "Write a story about space exploration:" \
  --max-tokens 512 \
  --temperature 0.9 \
  --top-k 40 \
  --top-p 0.95 \
  --stats
```

## 📖 Command Reference

### Main Commands

| Command | Description | Example |
|---------|-------------|---------|
| `run` | Run inference (default) | `rustlama run -m model.gguf -p "Hello"` |
| `models ls` | List cached models | `rustlama models ls --verbose` |
| `models pull` | Download model | `rustlama models pull TheBloke/Llama-2-7B-Chat-GGUF` |  
| `models rm` | Remove model | `rustlama models rm model-id` |
| `models du` | Disk usage | `rustlama models du` |
| `config` | Run YAML workflow | `rustlama config --file tasks.yml` |

### Options Reference

| Option | Short | Description | Default |
|--------|-------|-------------|---------|
| `--model` | `-m` | Model path or HF ID | Required |
| `--prompt` | `-p` | Input prompt | Required |
| `--max-tokens` | `-n` | Max tokens to generate | 1024 |
| `--temperature` | `-t` | Sampling temperature | 0.8 |
| `--top-k` |  | Top-k sampling | 40 |
| `--top-p` |  | Top-p sampling | 0.95 |
| `--ctx-size` | `-c` | Context size | 2048 |
| `--threads` | `-j` | Inference threads | Auto |
| `--stats` | `-s` | Show statistics | false |
| `--verbose` | `-v` | Verbose output | false |

### Local Model Usage

```bash
# Basic usage with local model
rustlama run --model model.gguf --prompt "Hello, world!"

# Advanced local model usage
rustlama run \
  --model llama-2-7b-chat.gguf \
  --prompt "Explain quantum computing in simple terms" \
  --max-tokens 1024 \
  --temperature 0.7 \
  --verbose
```

## 📖 Usage

### Basic Command Structure

```bash
rustlama [OPTIONS] --model <FILE> --prompt <TEXT>
```

### Options

| Option | Short | Description | Default |
|--------|-------|-------------|---------|
| `--model` | `-m` | Path to GGUF model file or Hugging Face model ID | Required |
| `--hf-filename` |  | Specific filename to download from HF model | Auto-detect |
| `--prompt` | `-p` | Input prompt for generation | Required |
| `--max-tokens` | `-n` | Maximum tokens to generate | 1024 |
| `--temperature` | `-t` | Sampling temperature (0.1-2.0) | 0.8 |
| `--top-k` |  | Top-k sampling parameter | 40 |
| `--top-p` |  | Top-p sampling parameter (0.0-1.0) | 0.95 |
| `--ctx-size` | `-c` | Context size in tokens | 2048 |
| `--threads` | `-j` | Number of inference threads | Auto-detect |
| `--stats` | `-s` | Show generation statistics | false |
| `--verbose` | `-v` | Enable verbose output | false |
| `--no-color` |  | Disable colored output | false |

### Examples

#### Simple Chat

```bash
rustlama run -m llama2-chat.gguf -p "What is the capital of France?"
```

#### Creative Writing

```bash
rustlama run \
  -m mistral-7b.gguf \
  -p "Once upon a time in a distant galaxy," \
  --temperature 1.2 \
  --max-tokens 300 \
  --stats
```

#### Technical Documentation

```bash
rustlama run \
  -m codellama-7b.gguf \
  -p "Write a Python function to calculate fibonacci:" \
  --temperature 0.2 \
  --top-k 20 \
  --verbose
```

#### High-Performance Setup

```bash
rustlama run \
  -m llama2-70b.gguf \
  -p "Analyze the economic implications of AI:" \
  --threads 16 \
  --ctx-size 4096 \
  --max-tokens 500
```

## 🛠️ Model Compatibility

RustLlama works with all GGUF format models, including:

- **LLaMA 2**: All sizes (7B, 13B, 70B)
- **Mistral**: 7B and variants  
- **CodeLlama**: All code-specialized models
- **Vicuna**: Chat-tuned models
- **And many more**: Any model in GGUF format

### Where to Get Models

- [Hugging Face](https://huggingface.co/models?library=gguf) - Search for GGUF models
- [TheBloke's Models](https://huggingface.co/TheBloke) - High-quality quantized models
- [Ollama Model Library](https://ollama.ai/library) - Curated model collection

## 🎉 Recent Updates

### v0.2.0 - Workflow Automation & File Output 🚀

**🎯 Major Features Added:**

- **YAML Workflow Automation**: Complete batch processing system with declarative configuration
- **File Output System**: All generated text automatically saved to specified files  
- **Selective Task Execution**: Run specific tasks or skip tasks by name with `--only-tasks` and `--skip-tasks`
- **Dry-Run Mode**: Preview workflow execution without running inference with `--dry-run`
- **Enhanced Error Handling**: Continue-on-error support for robust batch processing
- **Advanced Model Management**: Comprehensive Docker-like commands for model lifecycle

**📁 File Output Features:**

- Generated text automatically saved to files (`.txt`, `.md`, `.py`, etc.)
- Configurable output paths in YAML workflows  
- Perfect for content generation pipelines and batch processing
- Enables easy review of generated content later

**🐳 Docker-like CLI Enhancements:**

- Restructured command system: `run`, `models pull/ls/rm/du`
- Advanced model caching and deletion capabilities
- Improved command organization and comprehensive help system

### v0.1.1 - Major Stability & Performance Release

**🔧 Core Fixes:**

- **Fixed Logit Indexing Bug**: Resolved critical panic during token generation  
- **Enhanced Token Generation**: Increased default max tokens from 128 → 1024 (8x improvement)
- **Stable Autoregressive Generation**: Proper handling of prompt vs. generation phase logits

**🤗 Hugging Face Integration:**

- **Direct Model Downloads**: Seamless integration with HuggingFace Hub
- **Smart Caching**: Automatic model caching with `~/.cache/rustlama/models/`
- **Auto-detection**: Intelligent filename detection for GGUF models

**🚀 Performance Enhancements:**

- **Metal GPU Support**: Hardware acceleration on Apple Silicon (M1/M2/M3)
- **Optimized Inference**: Faster token generation with llama-cpp-2 bindings
- **Memory Efficiency**: Better memory management for large models

**🎨 User Experience:**

- **Rich Progress Indicators**: Beautiful loading animations and progress bars
- **Colored Output**: Syntax highlighting and colored text generation  
- **Verbose Mode**: Detailed logging for debugging and performance analysis
- **Comprehensive Error Handling**: Clear error messages and recovery suggestions

## ⚡ Performance Benchmarks

### Generation Speed (Apple M3 Max)

- **Llama-2-7B Q2_K**: ~45-60 tokens/second
- **Llama-2-7B Q4_K_M**: ~35-50 tokens/second  
- **Metal GPU Acceleration**: Automatic on Apple Silicon
- **Memory Usage**: Efficient with 2GB-6GB depending on model size

### Features Delivered ✅

- ✅ **Fixed Logit Indexing**: Stable autoregressive generation
- ✅ **1024+ Token Generation**: Extended from original 128 tokens
- ✅ **HuggingFace Integration**: Seamless model downloading and caching
- ✅ **Metal GPU Support**: Hardware acceleration on Apple Silicon
- ✅ **Docker-like CLI**: Restructured commands for better UX
- ✅ **YAML Workflows**: Complete automation system with file output
- ✅ **File Output System**: Automatic saving of generated content
- ✅ **Production Ready**: Stable and reliable for enterprise use

## 🗺️ Roadmap

### Completed in v0.1.1 ✅

- ✅ **Hugging Face Integration**: Direct model downloading from HF Hub
- ✅ **Extended Generation**: Up to 1024+ tokens by default
- ✅ **Fixed Core Issues**: Resolved logit indexing for stable generation  
- ✅ **GPU Acceleration**: Metal support for Apple Silicon
- ✅ **Enhanced UX**: Better progress indicators and error messages

### Completed in v0.1.2 ✅

- ✅ **Docker-like CLI**: Restructured commands (`run`, `models pull/ls/rm/du`)
- ✅ **Model Management**: Advanced model caching and deletion capabilities
- ✅ **Enhanced Usability**: Improved command organization and help system

### Completed in v0.2.0 ✅

- ✅ **YAML Workflow System**: Complete batch processing and automation
- ✅ **File Output System**: Automatic saving of generated text to files
- ✅ **Selective Execution**: Run specific tasks or skip tasks by name
- ✅ **Dry-Run Mode**: Preview operations before execution  
- ✅ **Advanced Error Handling**: Continue-on-error support for batch processing

### Next Release: v0.3.0 - Interactive Features

- 🔄 **Chat Mode**: Interactive multi-turn conversations with memory
- 🔄 **Session Management**: Conversation persistence and history
- 🔄 **Advanced Sampling**: Nucleus sampling and repetition penalties
- 🔄 **Configuration Files**: TOML-based settings management

### Future Releases

- 🎯 **API Server**: REST endpoints with OpenAI API compatibility
- 🎯 **Model Quantization**: Built-in quantization tools
- 🎯 **Multi-GPU Support**: Distributed inference across devices
- 🎯 **Plugin System**: Extensible architecture for custom functionality

## ⚡ Performance Tips

1. **Use appropriate quantization**: Q4_K_M offers good balance of speed and quality
2. **Set optimal threads**: Use `--threads` matching your CPU cores  
3. **Adjust context size**: Larger contexts use more memory but provide better coherence
4. **Temperature tuning**:
   - Low (0.1-0.3): Deterministic, factual responses
   - Medium (0.5-0.8): Balanced creativity and coherence  
   - High (1.0-2.0): Creative, diverse outputs

## 🔧 Configuration

### Environment Variables

- `RUSTLAMA_MODEL_PATH`: Default model directory
- `RUSTLAMA_NO_COLOR`: Disable colored output (set to `1`)

### Config File Support (Coming Soon)

```toml
# ~/.config/rustlama/config.toml
default_model = "/path/to/your/favorite/model.gguf"
default_temperature = 0.7
default_max_tokens = 200
```

---

**Made with ❤️ by [Sangam Biradar](https://github.com/sangam14) and the RustLlama community**

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🤝 Contributing

We welcome contributions! Please see our [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Development Setup

```bash
git clone https://github.com/sangam14/rustlama.git
cd rustlama
cargo build --release
cargo test
```

## 🙏 Acknowledgments

- [llama.cpp](https://github.com/ggerganov/llama.cpp) - The underlying inference engine
- [llama-cpp-rs](https://github.com/utilityai/llama-cpp-rs) - Rust bindings for llama.cpp  
- The Rust community for excellent tooling and libraries

## 📚 Related Projects

- [llama.cpp](https://github.com/ggerganov/llama.cpp) - Original C++ implementation
- [Ollama](https://github.com/jmorganca/ollama) - Easy model management
- [LocalAI](https://github.com/go-skynet/LocalAI) - Self-hosted AI API

---

Happy inferencing! 🦙✨
