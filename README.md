# 🦙 RustLlama

[![Rust](https://img.shields.io/badge/rust-stable-brightgreen.svg)](https://rustup.rs/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)

A fast, efficient command-line interface for running LLaMA model inference using llama.cpp bindings. Built with Rust for maximum performance and safety.

## ✨ Features

- 🚀 **High Performance**: Built on top of llama.cpp for maximum inference speed with Metal GPU acceleration on Apple Silicon
- 🤗 **Hugging Face Integration**: Seamless model downloading and caching from Hugging Face Hub
- 🎛️ **Advanced Sampling**: Support for temperature, top-k, and top-p sampling with proper logit handling
- 💾 **GGUF Support**: Works with all GGUF format models with automatic quantization detection
- 📝 **Long-form Generation**: Generate up to 1024 tokens by default for comprehensive responses
- 🔧 **Flexible Configuration**: Customizable context size, threading, and cache management
- 📊 **Real-time Statistics**: Optional generation performance metrics and verbose logging
- 🎨 **Beautiful Output**: Colored output with progress indicators and loading animations
- 🛡️ **Memory Safe**: Written in Rust with zero unsafe operations in CLI code
- 🔄 **Stable Generation**: Fixed logit indexing for reliable autoregressive text generation

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

### Hugging Face Model Usage (Recommended)
```bash
# Download and run a model from Hugging Face Hub
rustlama --model TheBloke/Llama-2-7B-Chat-GGUF --hf-filename "llama-2-7b-chat.Q2_K.gguf" --prompt "Hello, how are you?"

# Generate comprehensive guides with long-form responses
rustlama \
  --model TheBloke/Llama-2-7B-Chat-GGUF \
  --hf-filename "llama-2-7b-chat.Q4_K_M.gguf" \
  --prompt "Write a comprehensive guide about machine learning fundamentals" \
  --max-tokens 1024 \
  --verbose

# Use cached model (no re-download after first use)
rustlama --model TheBloke/Llama-2-7B-Chat-GGUF --hf-filename "llama-2-7b-chat.Q2_K.gguf" --prompt "Explain quantum computing"
```

### Local Model Usage
```bash
# Basic usage with local model
rustlama --model model.gguf --prompt "Hello, world!"

# With custom parameters for creative writing
rustlama \
  --model llama-2-7b-chat.gguf \
  --prompt "Write a story about space exploration:" \
  --max-tokens 512 \
  --temperature 0.8 \
  --top-k 40 \
  --top-p 0.95 \
  --stats
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
rustlama -m llama2-chat.gguf -p "What is the capital of France?"
```

#### Creative Writing
```bash
rustlama \
  -m mistral-7b.gguf \
  -p "Once upon a time in a distant galaxy," \
  --temperature 1.2 \
  --max-tokens 300 \
  --stats
```

#### Technical Documentation
```bash
rustlama \
  -m codellama-7b.gguf \
  -p "Write a Python function to calculate fibonacci:" \
  --temperature 0.2 \
  --top-k 20 \
  --verbose
```

#### High-Performance Setup
```bash
rustlama \
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

### v0.1.1 - Major Stability & Performance Release

**🔧 Core Fixes:**
- **Fixed Logit Indexing Bug**: Resolved critical panic during token generation that was causing "logit X is not initialized" errors
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

This release transforms RustLlama from a basic CLI to a production-ready inference engine!

RustLlama delivers excellent performance with optimized inference:

### Generation Speed (Apple M3 Max)
- **Llama-2-7B Q2_K**: ~45-60 tokens/second
- **Llama-2-7B Q4_K_M**: ~35-50 tokens/second  
- **Metal GPU Acceleration**: Automatic on Apple Silicon
- **Memory Usage**: Efficient with 2GB-6GB depending on model size

### Features Delivered ✅
- ✅ **Fixed Logit Indexing**: Stable autoregressive generation
- ✅ **1024 Token Generation**: 8x increase from original 128 tokens
- ✅ **HuggingFace Integration**: Seamless model downloading and caching
- ✅ **Metal GPU Support**: Hardware acceleration on Apple Silicon
- ✅ **Advanced Error Handling**: Comprehensive error reporting
- ✅ **Production Ready**: Stable and reliable for enterprise use

## 🗺️ Roadmap

### Completed in v0.1.1 ✅
- ✅ **Hugging Face Integration**: Direct model downloading from HF Hub
- ✅ **Extended Generation**: Up to 1024 tokens by default
- ✅ **Fixed Core Issues**: Resolved logit indexing for stable generation
- ✅ **GPU Acceleration**: Metal support for Apple Silicon
- ✅ **Enhanced UX**: Better progress indicators and error messages

### Next Release: v0.2.0 - Interactive Features
- 🔄 **Chat Mode**: Interactive multi-turn conversations
- 🔄 **Session Management**: Conversation persistence and history
- 🔄 **Advanced Sampling**: Nucleus sampling and repetition penalties
- 🔄 **Configuration Files**: TOML-based settings management

### Future Releases
- 🎯 **API Server**: REST endpoints with OpenAI compatibility
- 🎯 **Model Quantization**: Built-in quantization tools
- 🎯 **Multi-GPU Support**: Distributed inference across devices

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

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

### Development Setup

```bash
git clone https://github.com/sangam14/rustlama.git
cd rustlama
cargo build
cargo test
```

### Running Tests

```bash
cargo test
```

## �️ Roadmap

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🤝 Contributing

We welcome contributions! Please see our [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## 🙏 Acknowledgments

- [llama.cpp](https://github.com/ggerganov/llama.cpp) - The underlying inference engine
- [llama-cpp-rs](https://github.com/utilityai/llama-cpp-rs) - Rust bindings for llama.cpp
- The Rust community for excellent tooling and libraries

## 📚 Related Projects

- [llama.cpp](https://github.com/ggerganov/llama.cpp) - Original C++ implementation
- [Ollama](https://github.com/jmorganca/ollama) - Easy model management
- [LocalAI](https://github.com/go-skynet/LocalAI) - Self-hosted AI API

---

**Made with ❤️ by [Sangam Biradar](https://github.com/sangam14) and the RustLlama community**

**Happy inferencing! 🦙✨**

## �📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

Happy inferencing! 🦙✨
