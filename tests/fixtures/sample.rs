// Sample Rust file for testing

use std::collections::HashMap;
use std::fmt::Display;

/// A user in the system
pub struct User {
    pub id: u64,
    pub name: String,
    pub email: String,
}

impl User {
    /// Creates a new user
    pub fn new(id: u64, name: String, email: String) -> Self {
        Self { id, name, email }
    }

    /// Gets the user's display name
    pub fn display_name(&self) -> &str {
        &self.name
    }
}

impl Display for User {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "User {{ id: {}, name: {} }}", self.id, self.name)
    }
}

/// User role enumeration
pub enum UserRole {
    Admin,
    Moderator,
    Member,
    Guest,
}

impl UserRole {
    /// Checks if the role has admin privileges
    pub fn is_admin(&self) -> bool {
        matches!(self, UserRole::Admin)
    }
}

/// A trait for entities that can be validated
pub trait Validatable {
    /// Validates the entity
    fn validate(&self) -> Result<(), String>;

    /// Checks if the entity is valid
    fn is_valid(&self) -> bool {
        self.validate().is_ok()
    }
}

impl Validatable for User {
    fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("Name cannot be empty".to_string());
        }
        if !self.email.contains('@') {
            return Err("Invalid email format".to_string());
        }
        Ok(())
    }
}

/// Type alias for user ID
pub type UserId = u64;

/// Type alias for a user map
pub type UserMap = HashMap<UserId, User>;

/// User service module
pub mod user_service {
    use super::*;

    /// Service for managing users
    pub struct UserService {
        users: UserMap,
    }

    impl UserService {
        /// Creates a new user service
        pub fn new() -> Self {
            Self {
                users: HashMap::new(),
            }
        }

        /// Adds a user to the service
        pub fn add_user(&mut self, user: User) {
            self.users.insert(user.id, user);
        }

        /// Gets a user by ID
        pub fn get_user(&self, id: UserId) -> Option<&User> {
            self.users.get(&id)
        }
    }

    impl Default for UserService {
        fn default() -> Self {
            Self::new()
        }
    }
}

/// Maximum number of users
pub const MAX_USERS: usize = 1000;

/// Default user name
pub const DEFAULT_NAME: &str = "Anonymous";

/// Global user counter
pub static mut USER_COUNTER: u64 = 0;

/// Increments and returns the user counter
///
/// # Safety
/// This function is unsafe because it mutates a static variable
pub unsafe fn next_user_id() -> u64 {
    USER_COUNTER += 1;
    USER_COUNTER
}

/// Helper function to process a user
pub fn process_user(user: &User) -> String {
    format!("Processing user: {}", user.display_name())
}

/// Helper function to validate email
pub fn validate_email(email: &str) -> bool {
    email.contains('@') && email.contains('.')
}

/// A macro for creating users
macro_rules! create_user {
    ($id:expr, $name:expr, $email:expr) => {
        User::new($id, $name.to_string(), $email.to_string())
    };
}

// TODO: Add more validation
// FIXME: Handle edge cases

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_creation() {
        let user = create_user!(1, "John", "john@example.com");
        assert_eq!(user.id, 1);
        assert_eq!(user.name, "John");
    }
}
