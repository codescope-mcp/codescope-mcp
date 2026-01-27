; Struct definitions
(struct_item
  name: (type_identifier) @name) @definition.struct

; Enum definitions
(enum_item
  name: (type_identifier) @name) @definition.enum

; Trait definitions
(trait_item
  name: (type_identifier) @name) @definition.trait

; Impl blocks
(impl_item
  type: (type_identifier) @name) @definition.impl

; Impl blocks with trait
(impl_item
  trait: (type_identifier)
  type: (type_identifier) @name) @definition.impl

; Function definitions (top-level)
(function_item
  name: (identifier) @name) @definition.function

; Method definitions in impl blocks
(impl_item
  body: (declaration_list
    (function_item
      name: (identifier) @name) @definition.method))

; Trait method signatures
(trait_item
  body: (declaration_list
    (function_signature_item
      name: (identifier) @name) @definition.method))

; Trait method with default implementation
(trait_item
  body: (declaration_list
    (function_item
      name: (identifier) @name) @definition.method))

; Type alias definitions
(type_item
  name: (type_identifier) @name) @definition.type_alias

; Module definitions
(mod_item
  name: (identifier) @name) @definition.module

; Const definitions
(const_item
  name: (identifier) @name) @definition.const

; Static definitions
(static_item
  name: (identifier) @name) @definition.static

; Macro definitions
(macro_definition
  name: (identifier) @name) @definition.macro
