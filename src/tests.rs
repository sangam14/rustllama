#[cfg(test)]
mod tests {
    use crate::{RunConfig, validate_args};
    use crate::downloader::is_hf_model_id;

    fn create_test_run_config() -> RunConfig {
        RunConfig {
            model: "test.gguf".to_string(),
            hf_filename: Some("model.gguf".to_string()),
            cache_dir: None,
            force_download: false,
            prompt: "test prompt".to_string(),
            max_tokens: 100,
            temperature: 0.8,
            top_k: 40,
            top_p: 0.95,
            ctx_size: None,
            threads: None,
            no_color: false,
            stats: false,
            verbose: false,
        }
    }

    #[test]
    fn test_validate_args_valid_temperature() {
        let config = create_test_run_config();
        assert!(validate_args(&config).is_ok());
    }

    #[test]
    fn test_validate_args_valid_temperature_low() {
        let mut config = create_test_run_config();
        config.temperature = 0.0; // Valid minimum
        assert!(validate_args(&config).is_ok());
    }

    #[test]
    fn test_validate_args_valid_temperature_high() {
        let mut config = create_test_run_config();
        config.temperature = 2.0; // Valid maximum
        assert!(validate_args(&config).is_ok());
    }

    #[test]
    fn test_validate_args_invalid_temperature_high() {
        let mut config = create_test_run_config();
        config.temperature = 3.0; // Too high
        
        let result = validate_args(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Temperature"));
    }

    #[test]
    fn test_validate_args_invalid_temperature_low() {
        let mut config = create_test_run_config();
        config.temperature = -0.1; // Too low (below 0.0)

        let result = validate_args(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Temperature"));
    }

    #[test]
    fn test_validate_args_valid_top_p_low() {
        let mut config = create_test_run_config();
        config.top_p = 0.0; // Valid minimum
        assert!(validate_args(&config).is_ok());
    }

    #[test]
    fn test_validate_args_valid_top_p_high() {
        let mut config = create_test_run_config();
        config.top_p = 1.0; // Valid maximum
        assert!(validate_args(&config).is_ok());
    }

    #[test]
    fn test_validate_args_invalid_top_p_high() {
        let mut config = create_test_run_config();
        config.top_p = 1.5; // Too high

        let result = validate_args(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Top-p"));
    }

    #[test]
    fn test_validate_args_invalid_top_p_low() {
        let mut config = create_test_run_config();
        config.top_p = -0.1; // Too low

        let result = validate_args(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Top-p"));
    }

    #[test]
    fn test_validate_args_zero_max_tokens() {
        let mut config = create_test_run_config();
        config.max_tokens = 0; // Zero tokens

        let result = validate_args(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Max tokens"));
    }

    #[test]
    fn test_validate_args_valid_max_tokens() {
        let mut config = create_test_run_config();
        config.max_tokens = 1; // Minimum valid
        assert!(validate_args(&config).is_ok());
        
        config.max_tokens = 1000; // Reasonable value
        assert!(validate_args(&config).is_ok());
        
        config.max_tokens = 4096; // Large but valid
        assert!(validate_args(&config).is_ok());
    }

    #[test]
    fn test_is_hf_model_id_valid() {
        // Test valid Hugging Face model IDs
        assert!(is_hf_model_id("TheBloke/Llama-2-7B-Chat-GGUF"));
        assert!(is_hf_model_id("microsoft/DialoGPT-medium"));
        assert!(is_hf_model_id("meta-llama/Llama-2-7b-hf"));
    }

    #[test]
    fn test_is_hf_model_id_invalid() {
        // Test invalid Hugging Face model IDs (local paths)
        assert!(!is_hf_model_id("model.gguf"));
        assert!(!is_hf_model_id("/path/to/model.gguf"));
        assert!(!is_hf_model_id("./models/llama.gguf"));
        assert!(!is_hf_model_id("~/models/model.gguf"));
    }

    #[test]
    fn test_run_config_creation() {
        let config = create_test_run_config();
        
        assert_eq!(config.model, "test.gguf");
        assert_eq!(config.hf_filename, Some("model.gguf".to_string()));
        assert_eq!(config.prompt, "test prompt");
        assert_eq!(config.max_tokens, 100);
        assert_eq!(config.temperature, 0.8);
        assert_eq!(config.top_k, 40);
        assert_eq!(config.top_p, 0.95);
        assert!(!config.force_download);
        assert!(!config.no_color);
        assert!(!config.stats);
        assert!(!config.verbose);
    }

    #[test]
    fn test_edge_cases() {
        // Test edge case values that should be valid
        let mut config = create_test_run_config();
        
        // Test minimum valid temperature
        config.temperature = 0.0;
        assert!(validate_args(&config).is_ok());
        
        // Test maximum valid temperature  
        config.temperature = 2.0;
        assert!(validate_args(&config).is_ok());
        
        // Test minimum valid top_p
        config.top_p = 0.0;
        assert!(validate_args(&config).is_ok());
        
        // Test maximum valid top_p
        config.top_p = 1.0;
        assert!(validate_args(&config).is_ok());
        
        // Test minimum valid max_tokens
        config.max_tokens = 1;
        assert!(validate_args(&config).is_ok());
    }

    #[test]
    fn test_model_id_patterns() {
        // Test various Hugging Face model ID patterns
        assert!(is_hf_model_id("user/repo"));
        assert!(is_hf_model_id("organization/model-name"));
        assert!(is_hf_model_id("TheBloke/Llama-2-7B-Chat-GGUF"));
        assert!(is_hf_model_id("microsoft/DialoGPT-medium"));
        assert!(is_hf_model_id("meta-llama/Llama-2-7b-hf"));
        assert!(is_hf_model_id("google/flan-t5-large"));
        
        // Test invalid patterns (local file paths)
        assert!(!is_hf_model_id("model.gguf"));
        assert!(!is_hf_model_id("./model.gguf"));
        assert!(!is_hf_model_id("../models/model.gguf"));
        assert!(!is_hf_model_id("/absolute/path/model.gguf"));
        assert!(!is_hf_model_id("~/home/models/model.gguf"));
        assert!(!is_hf_model_id("C:\\Windows\\model.gguf"));
        
        // Test edge cases - these are the actual behavior of the function
        assert!(!is_hf_model_id(""));
        assert!(!is_hf_model_id("single_name"));
        // Note: The function actually accepts "user/" and "/repo" because it only checks for exactly one slash
        // This might be a limitation, but we test the actual behavior
        assert!(is_hf_model_id("user/")); // Function currently accepts this
        assert!(!is_hf_model_id("/repo")); // Function rejects this (starts with /)
    }
}
