/// Email and password validation module
///
/// Provides validation functions for user input according to security best practices.
use regex::Regex;
use std::sync::OnceLock;

/// Minimum password length
pub const MIN_PASSWORD_LENGTH: usize = 8;

/// Email validation regex pattern (simplified RFC 5322)
static EMAIL_REGEX: OnceLock<Regex> = OnceLock::new();

fn get_email_regex() -> &'static Regex {
    EMAIL_REGEX
        .get_or_init(|| Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap())
}

/// Validation errors
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationError {
    /// Email is invalid or empty
    InvalidEmail(String),
    /// Password is too short
    PasswordTooShort {
        min_length: usize,
        actual_length: usize,
    },
    /// Password is missing required character types
    PasswordMissingRequirements(Vec<String>),
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::InvalidEmail(email) => {
                write!(f, "Invalid email address: {}", email)
            }
            ValidationError::PasswordTooShort {
                min_length,
                actual_length,
            } => write!(
                f,
                "Password too short: {} characters (minimum: {})",
                actual_length, min_length
            ),
            ValidationError::PasswordMissingRequirements(reqs) => {
                write!(f, "Password missing requirements: {}", reqs.join(", "))
            }
        }
    }
}

impl std::error::Error for ValidationError {}

/// Validates an email address
///
/// # Arguments
///
/// * `email` - The email address to validate
///
/// # Returns
///
/// * `Ok(())` - Email is valid
/// * `Err(ValidationError::InvalidEmail)` - Email is invalid
///
/// # Examples
///
/// ```
/// use blackjack_core::validation::validate_email;
///
/// assert!(validate_email("user@example.com").is_ok());
/// assert!(validate_email("invalid").is_err());
/// assert!(validate_email("").is_err());
/// ```
pub fn validate_email(email: &str) -> Result<(), ValidationError> {
    if email.is_empty() {
        return Err(ValidationError::InvalidEmail("Email is empty".to_string()));
    }

    if !get_email_regex().is_match(email) {
        return Err(ValidationError::InvalidEmail(
            "Email format is invalid".to_string(),
        ));
    }

    Ok(())
}

/// Validates a password according to security requirements
///
/// Requirements:
/// - Minimum 8 characters
/// - At least one uppercase letter
/// - At least one lowercase letter
/// - At least one digit
/// - At least one special character
///
/// # Arguments
///
/// * `password` - The password to validate
///
/// # Returns
///
/// * `Ok(())` - Password meets all requirements
/// * `Err(ValidationError)` - Password fails validation
///
/// # Examples
///
/// ```
/// use blackjack_core::validation::validate_password;
///
/// assert!(validate_password("MyP@ssw0rd").is_ok());
/// assert!(validate_password("weak").is_err());
/// assert!(validate_password("NoNumber!").is_err());
/// ```
pub fn validate_password(password: &str) -> Result<(), ValidationError> {
    if password.len() < MIN_PASSWORD_LENGTH {
        return Err(ValidationError::PasswordTooShort {
            min_length: MIN_PASSWORD_LENGTH,
            actual_length: password.len(),
        });
    }

    let mut missing_requirements = Vec::new();

    if !password.chars().any(|c| c.is_uppercase()) {
        missing_requirements.push("uppercase letter".to_string());
    }

    if !password.chars().any(|c| c.is_lowercase()) {
        missing_requirements.push("lowercase letter".to_string());
    }

    if !password.chars().any(|c| c.is_numeric()) {
        missing_requirements.push("digit".to_string());
    }

    if !password.chars().any(|c| !c.is_alphanumeric()) {
        missing_requirements.push("special character".to_string());
    }

    if !missing_requirements.is_empty() {
        return Err(ValidationError::PasswordMissingRequirements(
            missing_requirements,
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_email_valid() {
        assert!(validate_email("user@example.com").is_ok());
        assert!(validate_email("test.user+tag@subdomain.example.com").is_ok());
        assert!(validate_email("name123@test.co.uk").is_ok());
    }

    #[test]
    fn test_validate_email_invalid() {
        assert!(validate_email("").is_err());
        assert!(validate_email("invalid").is_err());
        assert!(validate_email("@example.com").is_err());
        assert!(validate_email("user@").is_err());
        assert!(validate_email("user@domain").is_err());
        assert!(validate_email("user name@example.com").is_err());
    }

    #[test]
    fn test_validate_password_valid() {
        assert!(validate_password("MyP@ssw0rd").is_ok());
        assert!(validate_password("Secure#Pass123").is_ok());
        assert!(validate_password("C0mpl3x!Pass").is_ok());
    }

    #[test]
    fn test_validate_password_too_short() {
        let result = validate_password("Sh0rt!");
        assert!(matches!(
            result,
            Err(ValidationError::PasswordTooShort { .. })
        ));
    }

    #[test]
    fn test_validate_password_no_uppercase() {
        let result = validate_password("myp@ssw0rd");
        assert!(matches!(
            result,
            Err(ValidationError::PasswordMissingRequirements(_))
        ));
        if let Err(ValidationError::PasswordMissingRequirements(reqs)) = result {
            assert!(reqs.contains(&"uppercase letter".to_string()));
        }
    }

    #[test]
    fn test_validate_password_no_lowercase() {
        let result = validate_password("MYP@SSW0RD");
        assert!(matches!(
            result,
            Err(ValidationError::PasswordMissingRequirements(_))
        ));
        if let Err(ValidationError::PasswordMissingRequirements(reqs)) = result {
            assert!(reqs.contains(&"lowercase letter".to_string()));
        }
    }

    #[test]
    fn test_validate_password_no_digit() {
        let result = validate_password("MyP@ssword");
        assert!(matches!(
            result,
            Err(ValidationError::PasswordMissingRequirements(_))
        ));
        if let Err(ValidationError::PasswordMissingRequirements(reqs)) = result {
            assert!(reqs.contains(&"digit".to_string()));
        }
    }

    #[test]
    fn test_validate_password_no_special() {
        let result = validate_password("MyPassw0rd");
        assert!(matches!(
            result,
            Err(ValidationError::PasswordMissingRequirements(_))
        ));
        if let Err(ValidationError::PasswordMissingRequirements(reqs)) = result {
            assert!(reqs.contains(&"special character".to_string()));
        }
    }

    #[test]
    fn test_validate_password_multiple_missing() {
        let result = validate_password("password");
        assert!(matches!(
            result,
            Err(ValidationError::PasswordMissingRequirements(_))
        ));
        if let Err(ValidationError::PasswordMissingRequirements(reqs)) = result {
            assert!(reqs.len() >= 2); // Missing uppercase, digit, and special
        }
    }
}
