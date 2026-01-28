-- Sample SQL file for testing CodeScope MCP
-- This file contains various SQL constructs for symbol detection

-- Table definition with columns
CREATE TABLE users (
    id INT PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    email VARCHAR(255) UNIQUE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Another table for testing
CREATE TABLE orders (
    order_id INT PRIMARY KEY,
    user_id INT REFERENCES users(id),
    total DECIMAL(10, 2),
    status VARCHAR(50)
);

-- View definition
CREATE VIEW active_users AS
SELECT id, name, email
FROM users
WHERE active = true;

-- Another view with join
CREATE VIEW user_orders AS
SELECT u.name, o.order_id, o.total
FROM users u
JOIN orders o ON u.id = o.user_id;

-- Index definitions
CREATE INDEX idx_users_email ON users(email);

CREATE INDEX idx_orders_user ON orders(user_id);

-- Function definition
CREATE FUNCTION get_user_count()
RETURNS INT
AS $$
    SELECT COUNT(*) FROM users;
$$;

-- Another function with parameters
CREATE FUNCTION get_user_by_email(user_email VARCHAR)
RETURNS TABLE(id INT, name VARCHAR, email VARCHAR)
AS $$
    SELECT id, name, email
    FROM users
    WHERE email = user_email;
$$;

-- Trigger definition
CREATE TRIGGER update_timestamp
BEFORE UPDATE ON users
FOR EACH ROW
EXECUTE FUNCTION update_modified_column();

-- Another trigger
CREATE TRIGGER audit_orders
AFTER INSERT ON orders
FOR EACH ROW
EXECUTE FUNCTION log_order_creation();

-- Stored procedure (using CREATE FUNCTION for PostgreSQL compatibility)
CREATE FUNCTION process_order(p_order_id INT)
RETURNS VOID
AS $$
BEGIN
    UPDATE orders SET status = 'processed' WHERE order_id = p_order_id;
END;
$$;

-- TODO: Add more validation functions
-- FIXME: Handle edge cases for NULL values

/*
 * Multi-line comment block
 * This is a sample SQL file for testing
 */

-- Simple queries for usage testing
SELECT * FROM users;
SELECT name, email FROM users WHERE id = 1;
INSERT INTO users (name, email) VALUES ('John Doe', 'john@example.com');
UPDATE users SET name = 'Jane Doe' WHERE id = 1;
DELETE FROM orders WHERE status = 'cancelled';
