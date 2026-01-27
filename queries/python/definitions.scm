; Function definitions (top-level)
(module
  (function_definition
    name: (identifier) @name) @definition.function)

; Decorated function definitions (top-level)
(module
  (decorated_definition
    definition: (function_definition
      name: (identifier) @name)) @definition.function)

; Class definitions
(class_definition
  name: (identifier) @name) @definition.class

; Decorated class definitions
(decorated_definition
  definition: (class_definition
    name: (identifier) @name)) @definition.class

; Constructor (__init__)
(class_definition
  body: (block
    (function_definition
      name: (identifier) @name
      (#eq? @name "__init__")) @definition.constructor))

; Decorated constructor
(class_definition
  body: (block
    (decorated_definition
      definition: (function_definition
        name: (identifier) @name
        (#eq? @name "__init__"))) @definition.constructor))

; Method definitions (excluding __init__)
(class_definition
  body: (block
    (function_definition
      name: (identifier) @name
      (#not-eq? @name "__init__")) @definition.method))

; Decorated method definitions (excluding __init__)
(class_definition
  body: (block
    (decorated_definition
      definition: (function_definition
        name: (identifier) @name
        (#not-eq? @name "__init__"))) @definition.method))

; Module-level variable assignments
(module
  (expression_statement
    (assignment
      left: (identifier) @name)) @definition.variable)
