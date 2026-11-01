| help shows the documentation (lib/helper over the embedded help.idx):
| words from reva.f and from libraries, one with a multi-line description,
| an unknown word, help/ and help alone; then every documented word is
| found by name (found, missing).
help dup
help see
help notebook
help needs
help nosuchword
help/ notebook
help
variable found   variable missing
: chk ( a n -- )  help-entry if 2drop 1 found +! else 1 missing +! then ;
' chk help-each  ." documented words found: " found @ . ." missing: " missing @ . cr
