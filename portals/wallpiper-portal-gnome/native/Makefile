CC ?= cc
GIR_SCANNER ?= g-ir-scanner
GIR_COMPILER ?= g-ir-compiler
CSTD_FLAGS ?= -std=gnu11 -Os -Wall -Wextra -ffunction-sections -fdata-sections -fno-asynchronous-unwind-tables -fno-unwind-tables

MUTTER_PKGS := libmutter-18 mutter-cogl-18 mutter-clutter-18 gobject-2.0 gio-unix-2.0 gbm egl libdrm
GI_PKGS := $(MUTTER_PKGS) gobject-introspection-1.0

LIB_CFLAGS := $(CSTD_FLAGS) -fPIC -Iinclude $(shell pkg-config --cflags $(GI_PKGS))
LIB_LIBS := $(shell pkg-config --libs $(MUTTER_PKGS))

TEST_PRODUCER_CFLAGS := $(CSTD_FLAGS) $(shell pkg-config --cflags gbm libdrm)
TEST_PRODUCER_LIBS := $(shell pkg-config --libs gbm libdrm)

GI_EXTRA_LIBS := $(shell pkg-config --libs $(GI_PKGS) | tr ' ' '\n' | sed -n 's/^-l//p')
GI_EXTRA_LIB_DIRS := $(shell pkg-config --libs $(GI_PKGS) | tr ' ' '\n' | grep '^-L')
GI_INCLUDE_PATH := $(shell pkg-config --variable=girdir gobject-introspection-1.0)
MUTTER_GIR_DIR := $(shell pkg-config --variable=girdir libmutter-18)

PREFIX ?= /usr
LIBDIR ?= $(PREFIX)/lib

TARGET_DIR := ../../../target/gnome
OBJ_DIR := $(TARGET_DIR)/objects

SO := $(TARGET_DIR)/libwallpiper-1.0.so
TEST_PRODUCER := $(TARGET_DIR)/wallpiper-test-producer
GIR := $(TARGET_DIR)/Wallpiper-1.0.gir
TYPELIB := $(TARGET_DIR)/Wallpiper-1.0.typelib

LIB_SRCS := \
	src/wallpiper_portal.c \
	src/wallpiper_test_actor.c \
	src/actor_stacking.c \
	src/capture_listener.c \
	src/ctl_listener.c \
	src/egl_import.c \
	src/error.c \
	src/monitor_geometry.c

LIB_OBJS := $(addprefix $(OBJ_DIR)/,$(notdir $(LIB_SRCS:.c=.o)))

GIR_SRCS := \
	include/wallpiper_portal.h \
	src/wallpiper_portal.c \
	src/wallpiper_test_actor.c

.PHONY: build install clean

build: $(SO) $(TEST_PRODUCER) $(TYPELIB)

$(SO): $(LIB_OBJS) | $(TARGET_DIR)
	$(CC) $(LIB_CFLAGS) -shared -Wl,--gc-sections -s -o $@ $(LIB_OBJS) $(LIB_LIBS)

$(OBJ_DIR)/%.o: src/%.c | $(OBJ_DIR)
	$(CC) $(LIB_CFLAGS) -c -o $@ $<

$(TEST_PRODUCER): $(OBJ_DIR)/test_producer.o | $(TARGET_DIR)
	$(CC) $(TEST_PRODUCER_CFLAGS) -Wl,--gc-sections -s -o $@ $(OBJ_DIR)/test_producer.o $(TEST_PRODUCER_LIBS)

$(OBJ_DIR)/test_producer.o: tools/test_producer.c | $(OBJ_DIR)
	$(CC) $(TEST_PRODUCER_CFLAGS) -c -o $@ $<

$(GIR): $(SO) $(GIR_SRCS) | $(TARGET_DIR)
	$(GIR_SCANNER) --quiet --no-libtool \
		--namespace=Wallpiper --nsversion=1.0 \
		--identifier-prefix=Wallpiper --symbol-prefix=wallpiper \
		--warn-all \
		--include=GObject-2.0 \
		-Iinclude \
		--cflags-begin $(shell pkg-config --cflags $(GI_PKGS)) --cflags-end \
		--add-include-path=$(MUTTER_GIR_DIR) --add-include-path=$(GI_INCLUDE_PATH) \
		-L$(TARGET_DIR) --library=wallpiper-1.0 \
		$(GI_EXTRA_LIB_DIRS) \
		$(addprefix --extra-library=,$(GI_EXTRA_LIBS)) \
		--output $@ \
		$(GIR_SRCS)

$(TYPELIB): $(GIR) | $(TARGET_DIR)
	$(GIR_COMPILER) $< --output $@ --includedir=$(MUTTER_GIR_DIR) --includedir=$(GI_INCLUDE_PATH)

install: build
	install -Dm755 $(SO) $(DESTDIR)$(LIBDIR)/libwallpiper-1.0.so
	install -Dm644 $(GIR) $(DESTDIR)$(PREFIX)/share/gir-1.0/Wallpiper-1.0.gir
	install -Dm644 $(TYPELIB) $(DESTDIR)$(LIBDIR)/girepository-1.0/Wallpiper-1.0.typelib

$(TARGET_DIR) $(OBJ_DIR):
	mkdir -p $@

clean:
	rm -rf $(TARGET_DIR)
