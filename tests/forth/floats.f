| x87 floats through lib/math/floats (registers are f64; see DESIGN.md 3.10)
needs math/floats ~floats
8 sigdig !
." arith  " 1 s>f 3 s>f f/ f.  7 s>f 2 s>f f* f.  10 s>f 4 s>f f- f.  fpi f. cr
." conv   " " 3.25" >float . f.  " -2.75" >float drop f>s .  1234567 s>f f>s . cr
." round  " " 2.5" >float drop fround f>s .  " -2.5" >float drop fint f>s .  " 2.7" >float drop ffloor f>s . cr
." cmp    " 1 s>f 2 s>f f< .  2 s>f 1 s>f f> .  3 s>f 3 s>f f= . cr
." func   " 2 s>f fsqrt f.  30 s>f deg>rad fsin f.  60 s>f deg>rad fcos f.  1 s>f fatan rad>deg f.  2 s>f fln f.  1 s>f fexp f. cr
." memory " fpi pad f!  pad 10 + off  pad 8 + w@ .x  pad f@ f.  fpi pad f!8 pad f@8 f.  " 0.1" >float drop pad f!4 pad f@4 f. cr
