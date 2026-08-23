#pragma once

typedef void (*wp_shutdown_fn)(void);

void wp_reap_children_forever(void);
void wp_install_shutdown_handler(wp_shutdown_fn on_shutdown);
void wp_ignore_sigpipe(void);
