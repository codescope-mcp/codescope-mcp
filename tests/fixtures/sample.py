# Sample Python file for testing

from typing import List, Optional


class UserService:
    """Service for managing users."""

    def __init__(self, db_url: str):
        """Initialize the service with database URL."""
        self.db_url = db_url
        self.users: List[User] = []
        print("UserService initialized")

    def add_user(self, user: "User") -> None:
        """Add a user to the service."""
        self.users.append(user)

    def find_user(self, user_id: int) -> Optional["User"]:
        """Find a user by ID."""
        for user in self.users:
            if user.id == user_id:
                return user
        return None

    def get_all_users(self) -> List["User"]:
        """Get all users."""
        return self.users


class User:
    """Represents a user."""

    def __init__(self, id: int, name: str, email: str):
        self.id = id
        self.name = name
        self.email = email


def process_user(user: User) -> None:
    """Process a single user."""
    print(f"Processing user: {user.name}")


def validate_email(email: str) -> bool:
    """Validate email format."""
    return "@" in email


@staticmethod
def format_name(first_name: str, last_name: str) -> str:
    """Format full name from first and last name."""
    return f"{first_name} {last_name}"


class Logger:
    """Simple logger class."""

    def __init__(self, prefix: str):
        self.prefix = prefix

    @property
    def formatted_prefix(self) -> str:
        """Get formatted prefix."""
        return f"[{self.prefix}]"

    def log(self, message: str) -> None:
        """Log a message."""
        print(f"{self.formatted_prefix} {message}")

    @classmethod
    def create_default(cls) -> "Logger":
        """Create a logger with default prefix."""
        return cls("DEFAULT")


# Module-level variable
DEFAULT_CONFIG = {
    "api_url": "https://api.example.com",
    "timeout": 5000,
}

# TODO: Add more validation
# FIXME: Handle edge cases
