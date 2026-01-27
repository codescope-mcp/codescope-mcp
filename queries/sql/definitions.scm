; SQL definitions query

; Table definitions
; CREATE TABLE users (...)
(create_table
  (object_reference
    (identifier) @name)) @definition.table

; View definitions
; CREATE VIEW active_users AS ...
(create_view
  (object_reference
    (identifier) @name)) @definition.view

; Index definitions
; CREATE INDEX idx_users_email ON users(...)
; Note: index name is captured as column field in tree-sitter-sequel
(create_index
  column: (identifier) @name) @definition.index

; Function definitions
; CREATE FUNCTION get_user_count() ...
(create_function
  (object_reference
    (identifier) @name)) @definition.function

; Trigger definitions
; CREATE TRIGGER update_timestamp ...
(create_trigger
  (object_reference
    (identifier) @name)
  .
  [(keyword_before) (keyword_after)]) @definition.trigger

; Column definitions within CREATE TABLE
; id INT PRIMARY KEY, name VARCHAR(100), ...
(column_definition
  name: (identifier) @name) @definition.column
