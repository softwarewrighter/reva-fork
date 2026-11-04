| A notebook in a file of its own, with its own first lines (golden
| notebook-file; CoSy's ask 13).
needs ui/notebook
" mine.txt" notebook-file
:: " | my notebook" notebook-line  " 1 2 +" notebook-line ; is nb-examples
notebook
." saved: " " mine.txt" slurp type cr
." notebook.txt there: " " notebook.txt" slurp nip . cr
bye
