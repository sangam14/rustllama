# Hugging Face Model Download Examples

This document provides examples of using RustLlama's Hugging Face model download feature.

## Basic Usage

### Download and run a model
```bash
# Download a model from Hugging Face and run inference
rustlama --model microsoft/DialoGPT-medium --download --prompt "Hello, how are you today?"
```

### Specify a particular file
```bash
# Download a specific GGUF file from a model repository
rustlama --model huggingface/CodeBERTa-small-v1 --download --hf-filename "model-q4_0.gguf" --prompt "def hello_world():"
```

### Use custom cache directory
```bash
# Use a custom directory to cache models
rustlama --model microsoft/DialoGPT-medium --download --cache-dir "./my-models" --prompt "What is AI?"
```

### Force re-download
```bash
# Force re-download even if the model exists locally
rustlama --model microsoft/DialoGPT-medium --download --force-download --prompt "Explain machine learning"
```

## Model Detection

RustLlama can automatically detect if you're providing a Hugging Face model ID vs a local file path:

### Automatic Detection (works without --download flag)
```bash
# These will be detected as HF model IDs
rustlama --model microsoft/DialoGPT-medium --prompt "Hello"  # Will suggest using --download
rustlama --model huggingface/CodeBERTa-small-v1 --prompt "Code:"  # Will suggest using --download

# These will be treated as local file paths
rustlama --model "./model.gguf" --prompt "Hello"
rustlama --model "/path/to/model.gguf" --prompt "Hello"
rustlama --model "model.gguf" --prompt "Hello"
```

## Advanced Usage

### List available files
```bash
# Use verbose mode to see available GGUF files in a repository
rustlama --model microsoft/DialoGPT-medium --download --verbose --prompt "Hello"
```

### Using cached models
```bash
# After first download, you can use the model without --download
# (First run - downloads the model)
rustlama --model microsoft/DialoGPT-medium --download --prompt "First run"

# (Subsequent runs - uses cached model)
rustlama --model microsoft/DialoGPT-medium --prompt "Using cached model"
```

## Model Repository Structure

When downloading from Hugging Face, models are cached in:
- Default: `~/.cache/rustlama/models/`
- Custom: `<cache-dir>/models/`

The directory structure is:
```
~/.cache/rustlama/
├── models/
│   ├── microsoft--DialoGPT-medium/
│   │   └── model.gguf
│   └── huggingface--CodeBERTa-small-v1/
│       └── model-q4_0.gguf
```

## Error Handling

If a model is detected as a Hugging Face model ID but not found locally:
```bash
$ rustlama --model microsoft/DialoGPT-medium --prompt "Hello"
Error: Model not found locally. Use --download to download from Hugging Face.
Hint: If this is a Hugging Face model ID, use --download flag.
```

## Tips

1. **First time**: Always use `--download` for new models
2. **Subsequent runs**: Omit `--download` to use cached version
3. **Multiple files**: Use `--verbose` to see available options
4. **Specific files**: Use `--hf-filename` to download exact file names
5. **Storage**: Use `--cache-dir` to control where models are stored
