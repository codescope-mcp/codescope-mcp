// Sample JSX file for testing

import React from 'react';

class UserCard extends React.Component {
  constructor(props) {
    super(props);
    this.state = { expanded: false };
  }

  toggleExpand() {
    this.setState({ expanded: !this.state.expanded });
  }

  render() {
    const { user } = this.props;
    return (
      <div className="user-card">
        <h2>{user.name}</h2>
        {this.state.expanded && <p>{user.email}</p>}
        <button onClick={() => this.toggleExpand()}>Toggle</button>
      </div>
    );
  }
}

const UserList = ({ users }) => {
  return (
    <ul>
      {users.map(user => (
        <li key={user.id}>{user.name}</li>
      ))}
    </ul>
  );
};

function UserProfile({ user, onUpdate }) {
  const handleSubmit = (e) => {
    e.preventDefault();
    onUpdate(user);
  };

  return (
    <form onSubmit={handleSubmit}>
      <input value={user.name} />
      <button type="submit">Update</button>
    </form>
  );
}

// Exported arrow function component
export const Avatar = ({ src, alt }) => {
  return <img src={src} alt={alt} className="avatar" />;
};

// TODO: Add loading state
// FIXME: Handle missing user data

export { UserCard, UserList, UserProfile };
