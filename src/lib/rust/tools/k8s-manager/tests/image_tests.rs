//! Tests for image build and push functionality

#[cfg(test)]
mod tests {
    use std::process::Command;
    
    #[test]
    fn test_image_commands_help() {
        // Test that image commands are available in help
        let output = Command::new("cargo")
            .args(["run", "--", "image", "--help"])
            .output()
            .expect("Failed to execute command");
        
        let help_text = String::from_utf8_lossy(&output.stdout);
        
        assert!(output.status.success(), "Command should succeed");
        assert!(help_text.contains("build"), "Should have build command");
        assert!(help_text.contains("push"), "Should have push command");
        assert!(help_text.contains("build-and-push"), "Should have build-and-push command");
    }
    
    #[test]
    fn test_build_command_help() {
        let output = Command::new("cargo")
            .args(["run", "--", "image", "build", "--help"])
            .output()
            .expect("Failed to execute command");
        
        let help_text = String::from_utf8_lossy(&output.stdout);
        
        assert!(output.status.success(), "Command should succeed");
        assert!(help_text.contains("--context"), "Should have context option");
        assert!(help_text.contains("--dockerfile"), "Should have dockerfile option");
        assert!(help_text.contains("--tag"), "Should have tag option");
        assert!(help_text.contains("--build-arg"), "Should have build-arg option");
        assert!(help_text.contains("--no-cache"), "Should have no-cache option");
    }
}