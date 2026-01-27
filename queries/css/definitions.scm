; CSS selector and property definitions

; Class selector: .classname
(class_selector
  (class_name) @name) @definition.class_selector

; ID selector: #idname
(id_selector
  (id_name) @name) @definition.id_selector

; CSS custom property (variable) definition: --property-name
(declaration
  (property_name) @name
  (#match? @name "^--")) @definition.variable

; @keyframes animation definition
(keyframes_statement
  (keyframes_name) @name) @definition.keyframes
