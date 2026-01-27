; Link label references (in link_reference_definition)
(link_reference_definition
  (link_label) @usage)

; Note: Inline links like [text](url) are parsed by the inline grammar
; and would require a second parsing pass. For now, we only support
; link reference definitions which are visible in the block grammar.
