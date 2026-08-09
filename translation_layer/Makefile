CC ?= cc
CFLAGS ?= -O2 -Wall -Wextra -fPIC -fvisibility=hidden -D_GNU_SOURCE
DEPFLAGS := -MMD -MP
TARGET_DIR := ../target/release
OBJ_DIR := ../target/objects

SRCS := $(shell find ui_selection_hook swapchain_interpose -name '*.c')
OBJS := $(addprefix $(OBJ_DIR)/,$(SRCS:.c=.o))
DEPS := $(OBJS:.o=.d)
SO := $(TARGET_DIR)/libwallpiper-preload.so

.PHONY: all clean compile_commands

all: $(SO)

$(TARGET_DIR):
	mkdir -p $@

$(SO): $(OBJS) | $(TARGET_DIR)
	$(CC) $(CFLAGS) -shared -o $@ $(OBJS) -ldl -lpthread

$(OBJ_DIR)/%.o: %.c
	@mkdir -p $(dir $@)
	$(CC) $(CFLAGS) $(DEPFLAGS) -c -o $@ $<

clean:
	rm -rf $(OBJ_DIR) $(SO)

compile_commands: compile_commands.json

compile_commands.json: $(SRCS) Makefile
	@rm -f $@
	@echo '[' > $@
	@n=$(words $(SRCS)); i=0; \
	for f in $(SRCS); do \
		i=$$((i+1)); \
		obj="$(OBJ_DIR)/$${f%.c}.o"; \
		printf '  {\n    "directory": "%s",\n    "file": "%s",\n    "arguments": [' "$(CURDIR)" "$$f" >> $@; \
		first=1; \
		for a in $(CC) $(CFLAGS) -c "$$f" -o "$$obj"; do \
			[ $$first -eq 0 ] && printf ', ' >> $@; \
			first=0; \
			printf '"%s"' "$$a" >> $@; \
		done; \
		printf ']\n  }' >> $@; \
		[ $$i -lt $$n ] && printf ',\n' >> $@ || printf '\n' >> $@; \
	done
	@echo ']' >> $@

-include $(DEPS)
