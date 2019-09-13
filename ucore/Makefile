
#===============================================================================
# Paths and directories
#===============================================================================
LIBS_DIR     := libs
SRC_BOOT_DIR := boot
INC_BOOT_DIR := boot
SRC_KERN_DIR := kern/debug \
				kern/driver \
				kern/fs \
				kern/fs/devs \
				kern/fs/pipe \
				kern/fs/sfs \
				kern/fs/swap \
				kern/fs/vfs \
				kern/init \
				kern/libs \
				kern/mm \
				kern/process \
				kern/schedule \
				kern/sync \
				kern/syscall \
				kern/trap
INC_KERN_DIR := kern/debug \
				kern/driver \
				kern/fs \
				kern/fs/devs \
				kern/fs/pipe \
				kern/fs/sfs \
				kern/fs/swap \
				kern/fs/vfs \
				kern/libs \
				kern/mm \
				kern/process \
				kern/schedule \
				kern/sync \
				kern/syscall \
				kern/trap
INC_USER_DIR := user \
				user/libs
SRC_USER_DIR := user
LIB_USER_DIR := user/libs
SFS_ROOT_DIR := disk0
SFS_BIN_DIR  := $(SFS_ROOT_DIR)/bin
SRC_TOOL_DIR := tools
OBJ_DIR      := obj
BIN_DIR      := bin


#===============================================================================
# Commands
#===============================================================================
CC		:= gcc
GDB		:= gdb
CTYPE	:= c S
CFLAGS	:= -Wall -ggdb -m32 -Os -gstabs -fno-builtin -nostdinc
CFLAGS	+= $(shell $(CC) -fno-stack-protector -E -x c /dev/null >/dev/null 2>&1 && echo -fno-stack-protector)
CFLAGS	+= $(addprefix -I, $(LIBS_DIR))
BCFLAGS := $(CFLAGS)
BCFLAGS	+= $(addprefix -I, $(INC_BOOT_DIR))
KCFLAGS := $(CFLAGS)
KCFLAGS += $(addprefix -I, $(INC_KERN_DIR))
UCFLAGS := $(CFLAGS)
UCFLAGS += $(addprefix -I, $(INC_USER_DIR))

LD      := ld
BLDFLAGS:= -m elf_i386 -nostdlib -N -e start -Ttext 0x7C00
KLDFLAGS:= -m elf_i386 -nostdlib -T tools/kernel.ld
ULDFLAGS:= -m elf_i386 -nostdlib -T tools/user.ld

OBJCOPY := objcopy
OBJDUMP := objdump

#V		:= @
COPY	:= cp
MKDIR   := mkdir -p
MV		:= mv
RM		:= rm -f
SED 	:= sed


#===============================================================================
# Functions and variables
#===============================================================================
# make directories
define make_dir
	$(foreach str,\
			  $(addprefix $(OBJ_DIR)/,$(SRC_BOOT_DIR) $(SRC_KERN_DIR) $(SRC_USER_DIR) $(LIB_USER_DIR) $(LIBS_DIR)),\
			  $(shell $(MKDIR) $(str)))
	$(shell $(MKDIR) $(BIN_DIR))
	$(foreach str,\
			  $(addprefix $(BIN_DIR)/,$(SRC_USER_DIR)),\
			  $(shell $(MKDIR) $(str)))
	$(shell $(MKDIR) $(SFS_BIN_DIR))
endef

# list all source files in directories: (#src_dirs, #types)
get_src = $(filter $(addprefix %.,$(2)),$(wildcard $(addsuffix /*,$(1))))

# get .o obj files: (#obj_dirs, #src_files)
get_obj = $(addprefix $(1)/,$(addsuffix .o,$(basename $(2))))

# get bin files: (#bin_dirs, #src_files)
get_bin = $(addprefix $(1)/,$(basename $(2)))

# create obj target: (#obj_dir, #src_file, #cflags)
define do_create_obj_target
$(1)/$(basename $(2)).o: $(2)
	$(V)$(CC) $(3) -c $$< -o $$@
endef
create_obj_target = $(foreach line,$(2),$(eval $(call do_create_obj_target,$(1),$(line),$(3))))

# create bin target(for user programs): (#bin_dir, #obj_dir, #src_file, #lib_obj_file, #ldflags)
define do_create_bin_target
$(1)/$(basename $(3)): $(2)/$(basename $(3)).o $(4)
	$(V)$(LD) $(5) $$^ -o $$@
endef
create_bin_target = $(foreach line,$(3),$(eval $(call do_create_bin_target,$(1),$(2),$(line),$(4),$(5))))

# copy user bins to sfs: (#bin_file, #sfs_bin_dir)
define do_sfs_bin_copy
__sfs_bin_$(1): $(1)
	$(V)$(COPY) $(1) $(2)
endef
copy_files = $(foreach line,$(1),$(eval $(call do_sfs_bin_copy,$(line),$(2))))

$(eval $(call make_dir))
libs_src_files := $(call get_src,$(LIBS_DIR), $(CTYPE))
libs_obj_files := $(call get_obj,$(OBJ_DIR),$(libs_src_files))

boot_src_files := $(call get_src,$(SRC_BOOT_DIR),$(CTYPE))
boot_obj_files := $(call get_obj,$(OBJ_DIR),$(boot_src_files))
bootblock_obj  := $(OBJ_DIR)/bootblock
bootblock_bin  := $(BIN_DIR)/bootblock

kernel_src_files := $(call get_src,$(SRC_KERN_DIR),$(CTYPE))
kernel_obj_files := $(call get_obj,$(OBJ_DIR),$(kernel_src_files))
kernel_obj_files += $(libs_obj_files)
kernel_bin 		 := $(BIN_DIR)/kernel

user_bin_src_files := $(call get_src, $(SRC_USER_DIR), $(CTYPE))
user_lib_src_files := $(call get_src, $(LIB_USER_DIR), $(CTYPE))
user_lib_obj_files := $(call get_obj, $(OBJ_DIR), $(user_lib_src_files))
user_lib_obj_files += $(libs_obj_files)
user_bins  		   := $(call get_bin, $(BIN_DIR), $(user_bin_src_files))
sfs_bins		   := $(addprefix __sfs_bin_,$(user_bins))

sign_src_files := $(SRC_TOOL_DIR)/sign.c
sign_bin       := $(BIN_DIR)/sign

mksfs_src_files := $(SRC_TOOL_DIR)/mksfs.c
mksfs_bin       := $(BIN_DIR)/mksfs

UCOREIMG := $(BIN_DIR)/ucore.img
FSIMG    := $(BIN_DIR)/fs.img
SWAPIMG  := $(BIN_DIR)/swap.img


#===============================================================================
# Targets for ucore
#===============================================================================
# images
# ucore.img在硬盘上的分布：
# 		第一个扇区为引导代码(bootblock)，即bootloader
# 		第二个扇区开始为kernel代码
$(UCOREIMG): $(libs_obj_files) $(bootblock_bin) $(kernel_bin) $(SWAPIMG) $(FSIMG)
	$(V)echo
	$(V)echo "... making ucore ..."
	$(V)dd if=/dev/zero of=$@ count=10000
	$(V)dd if=$(bootblock_bin) of=$@ conv=notrunc
	$(V)dd if=$(kernel_bin) of=$@ seek=1 conv=notrunc

$(SWAPIMG):
	$(V)echo
	$(V)echo "... making swap ..."
	$(V)dd if=/dev/zero of=$@ bs=1M count=128

$(FSIMG): $(mksfs_bin) $(sfs_bins)
	$(V)echo
	$(V)echo "... making fs ..."
	$(V)$(COPY) $(BIN_DIR)/user/* $(SFS_BIN_DIR)
	$(V)dd if=/dev/zero of=$@ bs=1M count=128
	$(mksfs_bin) $@ $(SFS_ROOT_DIR)

$(call create_obj_target,$(OBJ_DIR),$(libs_src_files),$(CFLAGS))

# bootblock
$(bootblock_bin): $(bootblock_obj) $(sign_bin)
	$(V)echo
	$(V)echo "... making boot ..."
	$(V)$(OBJDUMP) -S $< > $@.asm
	$(V)$(OBJCOPY) -S -O binary $< $@.out
	$(V)$(sign_bin) $@.out $@

$(bootblock_obj): $(boot_obj_files)
	$(V)$(LD) $(BLDFLAGS) $^ -o $@

$(call create_obj_target,$(OBJ_DIR),$(boot_src_files),$(BCFLAGS))

# kernel
$(kernel_bin): $(kernel_obj_files)
	$(V)echo
	$(V)echo "... making kernel..."
	$(V)$(LD) $(KLDFLAGS) $^ -o $@
	$(V)$(OBJDUMP) -S $@ > $@.asm
	$(V)$(OBJDUMP) -t $@ | $(SED) '1,/SYMBOL TABLE/d; s/ .* / /; /^$$/d' > $@.sym

$(call create_obj_target,$(OBJ_DIR),$(kernel_src_files),$(KCFLAGS))

# user programs(bins)
$(call create_obj_target,$(OBJ_DIR),$(user_bin_src_files),$(UCFLAGS))
$(call create_obj_target,$(OBJ_DIR),$(user_lib_src_files),$(UCFLAGS))
$(call create_bin_target,$(BIN_DIR),$(OBJ_DIR),$(user_bin_src_files),$(user_lib_obj_files),$(ULDFLAGS))
$(call copy_files,$(user_bins),$(SFS_BIN_DIR))

# tools
$(sign_bin): $(sign_src_files)
	$(V)$(CC) -g -Wall -O2 $^ -o $@

$(mksfs_bin): $(mksfs_src_files)
	$(V)$(CC) -g -Wall -O2 $^ -o $@

.PHONY: test
test:
	@echo
	@echo $(libs_src_files)
	@echo $(libs_obj_files)
	@echo $(boot_src_files)
	@echo $(boot_obj_files)
	@echo $(kernel_src_files)
	@echo $(kernel_obj_files)
	@echo $(user_bin_src_files)
	@echo $(user_lib_src_files)
	@echo $(sfs_bins)
	@echo


#===============================================================================
# Targets for qemu
#===============================================================================
QEMU := qemu-system-i386
TERMINAL := xfce4-terminal

.PHONY: qemu
qemu: $(UCOREIMG)
	$(V)$(QEMU) -parallel stdio -hda $< -serial null

debug: $(UCOREIMG)
	$(V)$(QEMU) -S -s -parallel stdio -hda $< -serial null &
	$(V)sleep 2
	$(V)$(TERMINAL) -e "$(GDB) -q -x tools/gdbinit"


#===============================================================================
#  Clean
#===============================================================================
.PHONY: clean
clean:
	$(RM) -r $(OBJ_DIR) $(BIN_DIR) $(SFS_BIN_DIR)
