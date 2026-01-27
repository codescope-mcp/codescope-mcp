// Sample TypeScript file for testing

interface User {
  id: number;
  name: string;
  email: string;
}

class UserService {
  private users: User[] = [];

  constructor() {
    console.log("UserService initialized");
  }

  addUser(user: User): void {
    this.users.push(user);
  }

  findUser(id: number): User | undefined {
    return this.users.find(u => u.id === id);
  }

  getAllUsers(): User[] {
    return this.users;
  }
}

const createUser = (name: string, email: string): User => {
  return {
    id: Math.random(),
    name,
    email,
  };
};

function processUser(user: User): void {
  console.log(`Processing user: ${user.name}`);
}

export { User, UserService, createUser, processUser };
