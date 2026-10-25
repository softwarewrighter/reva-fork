| include: nested source files, ioerr on missing files
include tests/forth/inc/lib.f
from-lib . cr
include tests/forth/no-such-file.f
ioerr @ . cr
