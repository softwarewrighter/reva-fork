# Legacy 32-bit x86 build of Reva (deprecated; see ../DEPRECATED.md).
# Run from the repository root:   make -f legacy/Makefile [target]
# Forth sources (src/reva.f, lib/, tests/, tools/) stay at the root;
# nasm -Ilegacy/ resolves the cores' %include 'src/...' lines.

os=$(shell uname)
CROSS=0

ifeq ($(CROSS),1)
OSPATH=win
REVA=legacy/bin/$(OSPATH)/reva.exe
CORE=legacy/bin/$(OSPATH)/core.exe
BUILD=legacy/bin/$(OSPATH)/build.exe
OSCORE=legacy/src/windows.asm
OSNUM=0 -fwin32
LINKCMD=kernel32 legacy/src/revares.o
GCC=i686-pc-mingw32-gcc
else
ifeq ($(os),Linux)
OSPATH=lin
REVA=legacy/bin/$(OSPATH)/reva
CORE=legacy/bin/$(OSPATH)/core
BUILD=legacy/bin/$(OSPATH)/build
OSCORE=legacy/src/linux.asm
OSNUM=1 -felf
LINKCMD=dl -m32
GCC=gcc
else
OSPATH=mac
REVA=legacy/bin/$(OSPATH)/reva
CORE=legacy/bin/$(OSPATH)/core
BUILD=legacy/bin/$(OSPATH)/build
OSCORE=legacy/src/macosx.asm
OSNUM=2 -fmacho
LINKCMD=dl -Wl,-allow_stack_execute
GCC=gcc
endif
endif

$(REVA):  src/reva.f  $(BUILD) $(CORE)
	$(BUILD)

$(CORE): legacy/src/newcore.asm legacy/src/brieflz.asm  legacy/src/common.asm $(OSCORE)
	$(BUILD) core

$(BUILD): legacy/tools/build.f
	$(REVA) $<

test: $(REVA)
	$(BUILD) test

core: $(CORE)

reva: $(REVA)

bench: $(REVA)
	(cd bench; ../$(REVA) bench.f)

revagui:
	make -C fltk CROSS=$(CROSS)

# build core first:
bootstrap:
	nasm -Ox -o core.o -DOS=$(OSNUM) -Ilegacy/ legacy/src/newcore.asm
	$(GCC) -s -nostartfiles -o $(CORE) core.o -l$(LINKCMD)
	-strip $(CORE)
	-upx $(CORE)

ifeq ($(os),Darwin)
	cat legacy/src/zero >> $(CORE)
endif
	
	$(CORE) src/reva.f
	$(REVA) legacy/tools/build.f
