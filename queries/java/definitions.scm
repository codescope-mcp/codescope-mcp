; Class definitions
(class_declaration
  name: (identifier) @name) @definition.class

; Interface definitions
(interface_declaration
  name: (identifier) @name) @definition.interface

; Enum definitions
(enum_declaration
  name: (identifier) @name) @definition.enum

; Method definitions
(method_declaration
  name: (identifier) @name) @definition.method

; Constructor definitions
(constructor_declaration
  name: (identifier) @name) @definition.constructor

; Field definitions
(field_declaration
  declarator: (variable_declarator
    name: (identifier) @name)) @definition.field

; Annotation type definitions
(annotation_type_declaration
  name: (identifier) @name) @definition.annotation
