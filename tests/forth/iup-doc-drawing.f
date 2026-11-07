needs ui/gui
needs ui/cd
with~ ~ui  with~ ~iup  with~ ~cd

variable canvas  variable cdc
: paint callback
    cdc @ cd-activate drop
    CD_WHITE cd-background drop  cd-clear drop
    CD_BLUE cd-foreground drop  20 20 280 130 cd-line drop
    CD_RED cd-foreground drop  40 120 30 90 cd-box drop
    CD_BLACK cd-foreground drop  150 20 z" hello, CD" cd-text drop
    gui-default ;
: quit callback  gui-close ;

: picture ( -- )
    dialog[
      canvas[ " 300x150" attr: RASTERSIZE  dup canvas ! ]w
      " Quit" button[ action: quit ]w
    ]d " A picture" title
    gui-map
    cd-context-iup  canvas @ action: paint  cd-create-canvas cdc !
    show gui-main-loop
    cdc @ cd-kill-canvas ;
picture
