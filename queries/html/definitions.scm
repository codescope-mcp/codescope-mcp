; HTML element definitions
; Note: tree-sitter-html uses element -> start_tag -> tag_name structure

; Regular element with start tag
(element
  (start_tag
    (tag_name) @name)) @definition.element

; Self-closing element
(element
  (self_closing_tag
    (tag_name) @name)) @definition.element

; id attribute value
(attribute
  (attribute_name) @attr_name
  (quoted_attribute_value
    (attribute_value) @name)
  (#eq? @attr_name "id")) @definition.id

; class attribute value
(attribute
  (attribute_name) @attr_name
  (quoted_attribute_value
    (attribute_value) @name)
  (#eq? @attr_name "class")) @definition.class
