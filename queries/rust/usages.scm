; Identifiers (general usage)
(identifier) @usage

; Type identifiers
(type_identifier) @usage

; Field access expressions
(field_expression
  field: (field_identifier) @usage)

; Scoped identifiers (path expressions like std::io::Read)
(scoped_identifier
  name: (identifier) @usage)

; Scoped type identifiers
(scoped_type_identifier
  name: (type_identifier) @usage)
