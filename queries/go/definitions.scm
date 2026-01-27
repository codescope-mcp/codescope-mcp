; Function definitions
(function_declaration
  name: (identifier) @name) @definition.function

; Method definitions (with receiver)
(method_declaration
  name: (field_identifier) @name) @definition.method

; Struct definitions
(type_declaration
  (type_spec
    name: (type_identifier) @name
    type: (struct_type))) @definition.struct

; Interface definitions
(type_declaration
  (type_spec
    name: (type_identifier) @name
    type: (interface_type))) @definition.interface

; Type alias definitions (type A = B syntax)
(type_declaration
  (type_alias
    name: (type_identifier) @name)) @definition.type_alias

; Const definitions (single)
(const_declaration
  (const_spec
    name: (identifier) @name)) @definition.const

; Var definitions (single)
(var_declaration
  (var_spec
    name: (identifier) @name)) @definition.variable

; Short var declarations (:=)
(short_var_declaration
  left: (expression_list
    (identifier) @name)) @definition.variable
