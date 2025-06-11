Wikitext Grammar
<pre>
  document         := block*
  block            := heading | paragraph | hr | list | space
  paragraph        := inline (new_line inline)*

  heading          := (heading1 | heading2 | heading3 | heading4 | heading5 | heading6) new_line
  heading1         := "=" text "="
  heading2         := "==" text "=="
  heading3         := "===" text "==="
  heading4         := "====" text "===="
  heading5         := "=====" text "====="
  heading6         := "======" text "======"

  list             := (unordered_list | ordered_list | description_list)*
  unordered_list   := "*" inline (new_line "*" inline)*
  ordered_list     := "#" inline (new_line "#" inline)*
  description_list := ";" inline (colon inline)? (new_line colon inline)*

  inline           := (template | link | text)+

  template         := "{{" text_no_pipe ("|", document)* "}}"
  link             := "[[" text "]]"

  hr               := "-"{4,}
  colon            := ":"
  space            := new_line | " " | "\t"
  new_line         := "\n" | EOF
  text             := character+ where character ≠ '{', '[', '=', '*', '#', ':', '-', '\n'
  text_no_pipe     := character+ where character ≠ '|' and ≠ above specials
</pre>
