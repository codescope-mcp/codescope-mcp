// Sample Java file for testing
package com.example;

import java.util.List;
import java.util.ArrayList;

// User represents a user in the system
public class User {
    private int id;
    private String name;
    private String email;
    private UserRole role;

    // Constructor
    public User(int id, String name, String email) {
        this.id = id;
        this.name = name;
        this.email = email;
        this.role = UserRole.USER;
    }

    // Get display name
    public String getDisplayName() {
        if (name == null || name.isEmpty()) {
            return "Anonymous";
        }
        return name;
    }

    // Check if user is valid
    public boolean isValid() {
        return name != null && !name.isEmpty() && email != null && email.contains("@");
    }

    // Getters
    public int getId() {
        return id;
    }

    public String getName() {
        return name;
    }

    public String getEmail() {
        return email;
    }

    public UserRole getRole() {
        return role;
    }
}

// User role enumeration
enum UserRole {
    ADMIN,
    USER,
    GUEST
}

// Interface for validatable entities
interface Validatable {
    boolean isValid();
    void validate() throws ValidationException;
}

// Custom exception
class ValidationException extends Exception {
    public ValidationException(String message) {
        super(message);
    }
}

// User service class
class UserService {
    // Maximum number of users allowed
    private static final int MAX_USERS = 1000;
    private List<User> users;

    public UserService() {
        this.users = new ArrayList<>();
    }

    public void addUser(User user) {
        if (users.size() < MAX_USERS) {
            users.add(user);
        }
    }

    public User findUser(int id) {
        for (User user : users) {
            if (user.getId() == id) {
                return user;
            }
        }
        return null;
    }

    public int getUserCount() {
        return users.size();
    }
}

// Annotation type definition
@interface Deprecated {
    String since() default "";
    String forRemoval() default "false";
}

// TODO: Add more validation rules
// FIXME: Handle edge cases for empty strings

class Main {
    public static void main(String[] args) {
        // Create a new user
        User user = new User(1, "John Doe", "john@example.com");
        System.out.println(user.getDisplayName());

        // Create a user service
        UserService service = new UserService();
        service.addUser(user);

        // Find the user
        User found = service.findUser(1);
        if (found != null) {
            System.out.println("Found user: " + found.getName());
        }
    }
}
