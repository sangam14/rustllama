#[cfg(test)]
mod tests {
    use crate::{validate_args, Cli};

    fn create_test_cli() -> Cli {
        Cli {
            model: "test.gguf".to_string(),
            download: false,
            hf_filename: "model.gguf".to_string(),
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
        let cli = create_test_cli();
        assert!(validate_args(&cli).is_ok());
    }

    #[test]
    fn test_validate_args_invalid_temperature_high() {
        let mut cli = create_test_cli();
        cli.temperature = 3.0; // Too high
        
        let result = validate_args(&cli);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Temperature"));
    }

    #[test]
    fn test_validate_args_invalid_temperature_low() {
        let mut cli = create_test_cli();
        cli.temperature = -0.1; // Too low (below 0.0)

        let result = validate_args(&cli);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Temperature"));
    }

    #[test]
    fn test_validate_args_invalid_top_p() {
        let mut cli = create_test_cli();
        cli.top_p = 1.5; // Too high

        let result = validate_args(&cli);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Top-p"));
    }

    #[test]
    fn test_validate_args_zero_max_tokens() {
        let mut cli = create_test_cli();
        cli.max_tokens = 0; // Zero tokens

        let result = validate_args(&cli);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Max tokens"));
    }
}
