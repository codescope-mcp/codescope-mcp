; ATX Heading Level 1
(atx_heading
  (atx_h1_marker)
  (inline) @name) @definition.heading1

; ATX Heading Level 2
(atx_heading
  (atx_h2_marker)
  (inline) @name) @definition.heading2

; ATX Heading Level 3
(atx_heading
  (atx_h3_marker)
  (inline) @name) @definition.heading3

; ATX Heading Level 4
(atx_heading
  (atx_h4_marker)
  (inline) @name) @definition.heading4

; ATX Heading Level 5
(atx_heading
  (atx_h5_marker)
  (inline) @name) @definition.heading5

; ATX Heading Level 6
(atx_heading
  (atx_h6_marker)
  (inline) @name) @definition.heading6

; Fenced code block (only with language specification)
(fenced_code_block
  (info_string
    (language) @name)) @definition.code_block

; Link reference definition
(link_reference_definition
  (link_label) @name) @definition.link
