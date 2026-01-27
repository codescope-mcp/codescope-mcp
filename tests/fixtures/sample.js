// Sample JavaScript file for testing

class UserService {
  constructor() {
    this.users = [];
    console.log("UserService initialized");
  }

  addUser(user) {
    this.users.push(user);
  }

  findUser(id) {
    return this.users.find(u => u.id === id);
  }

  getAllUsers() {
    return this.users;
  }
}

const createUser = (name, email) => {
  return {
    id: Math.random(),
    name,
    email,
  };
};

function processUser(user) {
  console.log(`Processing user: ${user.name}`);
}

// Variable declarations with var
var globalConfig = {
  apiUrl: "https://api.example.com",
  timeout: 5000,
};

// Arrow function with export
export const formatName = (firstName, lastName) => {
  return `${firstName} ${lastName}`;
};

// Exported function declaration
export function validateEmail(email) {
  return email.includes("@");
}

// Exported class
export class Logger {
  constructor(prefix) {
    this.prefix = prefix;
  }

  log(message) {
    console.log(`[${this.prefix}] ${message}`);
  }
}

// TODO: Add more user validation
// FIXME: Handle edge cases

export { UserService, createUser, processUser };
