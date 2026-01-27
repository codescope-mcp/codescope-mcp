; Function declarations
(function_declaration
  name: (identifier) @name) @definition.function

; Class declarations
(class_declaration
  name: (type_identifier) @name) @definition.class

; Method definitions
(method_definition
  name: (property_identifier) @name) @definition.method

; Constructor
(method_definition
  name: (property_identifier) @name
  (#eq? @name "constructor")) @definition.constructor

; Interface declarations
(interface_declaration
  name: (type_identifier) @name) @definition.interface

; Enum declarations
(enum_declaration
  name: (identifier) @name) @definition.enum

; Variable declarations (const/let/var)
(lexical_declaration
  (variable_declarator
    name: (identifier) @name
    value: (_) @value)) @definition.variable

; Arrow function assigned to variable
(lexical_declaration
  (variable_declarator
    name: (identifier) @name
    value: (arrow_function))) @definition.arrow_function

; Type alias
(type_alias_declaration
  name: (type_identifier) @name) @definition.type_alias

; Export function declaration
(export_statement
  declaration: (function_declaration
    name: (identifier) @name)) @definition.function

; Export class declaration
(export_statement
  declaration: (class_declaration
    name: (type_identifier) @name)) @definition.class

; Export interface declaration
(export_statement
  declaration: (interface_declaration
    name: (type_identifier) @name)) @definition.interface

; Export enum declaration
(export_statement
  declaration: (enum_declaration
    name: (identifier) @name)) @definition.enum

; Export type alias
(export_statement
  declaration: (type_alias_declaration
    name: (type_identifier) @name)) @definition.type_alias

; Export variable declaration
(export_statement
  declaration: (lexical_declaration
    (variable_declarator
      name: (identifier) @name))) @definition.variable
