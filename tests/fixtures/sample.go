// Sample Go file for testing
package main

import (
	"fmt"
	"strings"
)

// MaxUsers is the maximum number of users allowed
const MaxUsers = 1000

// DefaultName is the default user name
const DefaultName = "Anonymous"

// globalCounter tracks the total number of users created
var globalCounter int

// User represents a user in the system
type User struct {
	ID    int
	Name  string
	Email string
}

// UserService manages user operations
type UserService struct {
	users []User
}

// Validatable is an interface for entities that can be validated
type Validatable interface {
	IsValid() bool
	Validate() error
}

// UserID is a type alias for user identifiers
type UserID = int

// NewUser creates a new User instance
func NewUser(id int, name, email string) *User {
	globalCounter++
	return &User{
		ID:    id,
		Name:  name,
		Email: email,
	}
}

// DisplayName returns the user's display name
func (u *User) DisplayName() string {
	if u.Name == "" {
		return DefaultName
	}
	return u.Name
}

// IsValid checks if the user is valid
func (u *User) IsValid() bool {
	return u.Name != "" && strings.Contains(u.Email, "@")
}

// Validate validates the user and returns an error if invalid
func (u *User) Validate() error {
	if u.Name == "" {
		return fmt.Errorf("name cannot be empty")
	}
	if !strings.Contains(u.Email, "@") {
		return fmt.Errorf("invalid email format")
	}
	return nil
}

// NewUserService creates a new UserService
func NewUserService() *UserService {
	return &UserService{
		users: make([]User, 0),
	}
}

// AddUser adds a user to the service
func (s *UserService) AddUser(user User) {
	s.users = append(s.users, user)
}

// FindUser finds a user by ID
func (s *UserService) FindUser(id int) *User {
	for i := range s.users {
		if s.users[i].ID == id {
			return &s.users[i]
		}
	}
	return nil
}

// processUser processes a user
func processUser(user *User) string {
	return fmt.Sprintf("Processing user: %s", user.DisplayName())
}

// validateEmail validates an email address
func validateEmail(email string) bool {
	return strings.Contains(email, "@") && strings.Contains(email, ".")
}

// TODO: Add more validation rules
// FIXME: Handle edge cases for empty strings

func main() {
	// Create a new user
	user := NewUser(1, "John Doe", "john@example.com")
	fmt.Println(user.DisplayName())

	// Create a user service
	service := NewUserService()
	service.AddUser(*user)

	// Process the user
	result := processUser(user)
	fmt.Println(result)
}
