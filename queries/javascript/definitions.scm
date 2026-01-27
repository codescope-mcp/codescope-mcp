; Function declarations
(function_declaration
  name: (identifier) @name) @definition.function

; Class declarations (JavaScript uses identifier, not type_identifier)
(class_declaration
  name: (identifier) @name) @definition.class

; Constructor (must come before method to avoid capturing as method)
(method_definition
  name: (property_identifier) @name
  (#eq? @name "constructor")) @definition.constructor

; Method definitions (excluding constructor)
(method_definition
  name: (property_identifier) @name
  (#not-eq? @name "constructor")) @definition.method

; Arrow function assigned to variable (must come before variable to capture arrow functions correctly)
(lexical_declaration
  (variable_declarator
    name: (identifier) @name
    value: (arrow_function))) @definition.arrow_function

; Variable declarations (const/let) - excludes arrow functions via processing order
(lexical_declaration
  (variable_declarator
    name: (identifier) @name
    value: (_) @value)) @definition.variable

; Variable declarations (var)
(variable_declaration
  (variable_declarator
    name: (identifier) @name
    value: (_) @value)) @definition.variable

; Export function declaration
(export_statement
  declaration: (function_declaration
    name: (identifier) @name)) @definition.function

; Export class declaration
(export_statement
  declaration: (class_declaration
    name: (identifier) @name)) @definition.class

; Export arrow function (lexical: const/let) - must come before export variable
(export_statement
  declaration: (lexical_declaration
    (variable_declarator
      name: (identifier) @name
      value: (arrow_function)))) @definition.arrow_function

; Export arrow function (var) - must come before export variable
(export_statement
  declaration: (variable_declaration
    (variable_declarator
      name: (identifier) @name
      value: (arrow_function)))) @definition.arrow_function

; Export variable declaration (lexical)
(export_statement
  declaration: (lexical_declaration
    (variable_declarator
      name: (identifier) @name))) @definition.variable

; Export variable declaration (var)
(export_statement
  declaration: (variable_declaration
    (variable_declarator
      name: (identifier) @name))) @definition.variable
