/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2026 Ethan Alexander
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in
 * all copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */

#include "tray.h"
#include "tray_internal.h"

#include <dbus/dbus.h>
#include <pthread.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <unistd.h>

#define SNI_PATH "/StatusNotifierItem"
#define MENU_PATH "/MenuBar"
#define SNI_IFACE "org.kde.StatusNotifierItem"
#define MENU_IFACE "com.canonical.dbusmenu"
#define PROPS_IFACE "org.freedesktop.DBus.Properties"
#define INTROSPECT_IFACE "org.freedesktop.DBus.Introspectable"
#define MENU_REFRESH_TIMEOUT_SEC 8
#define MENU_REFRESH_RETRY_INTERVAL_SEC 1
#define MENU_REFRESH_SETTLE_MS 80
#define WP_TRAY_PREWARM_MAX_SEC 60

typedef struct {
  pthread_mutex_t mutex;
  pthread_cond_t entries_cond;

  int32_t icon_width;
  int32_t icon_height;
  uint8_t *icon_argb;
  char tooltip[256];

  wp_tray_entries_t entries;
  uint64_t generation;
  uint32_t revision;
  bool refresh_in_progress;

  DBusConnection *conn;
  char well_known_name[128];
} wp_tray_state_t;

static wp_tray_state_t g_state;

static bool refresh_menu_from_win32(void);

static void register_status_notifier_item(void) {
  DBusMessage *msg = dbus_message_new_method_call(
      "org.kde.StatusNotifierWatcher", "/StatusNotifierWatcher",
      "org.kde.StatusNotifierWatcher", "RegisterStatusNotifierItem");
  if (!msg) {
    return;
  }
  const char *name = g_state.well_known_name;
  dbus_message_append_args(msg, DBUS_TYPE_STRING, &name, DBUS_TYPE_INVALID);

  DBusError err;
  dbus_error_init(&err);
  DBusMessage *reply =
      dbus_connection_send_with_reply_and_block(g_state.conn, msg, 1000, &err);
  dbus_message_unref(msg);

  if (!reply) {
    if (err.name &&
        strcmp(err.name, "org.freedesktop.DBus.Error.ServiceUnknown") != 0) {
      printf("tray: RegisterStatusNotifierItem failed: %s\n",
             err.message ? err.message : "unknown error");
    }
    dbus_error_free(&err);
    return;
  }
  dbus_message_unref(reply);
}

static DBusHandlerResult name_owner_changed_filter(DBusConnection *conn,
                                                   DBusMessage *msg,
                                                   void *user_data) {
  (void)conn;
  (void)user_data;

  if (!dbus_message_is_signal(msg, "org.freedesktop.DBus",
                              "NameOwnerChanged")) {
    return DBUS_HANDLER_RESULT_NOT_YET_HANDLED;
  }

  const char *name = NULL;
  const char *old_owner = NULL;
  const char *new_owner = NULL;
  DBusError err;
  dbus_error_init(&err);
  if (!dbus_message_get_args(msg, &err, DBUS_TYPE_STRING, &name,
                             DBUS_TYPE_STRING, &old_owner, DBUS_TYPE_STRING,
                             &new_owner, DBUS_TYPE_INVALID)) {
    dbus_error_free(&err);
    return DBUS_HANDLER_RESULT_NOT_YET_HANDLED;
  }

  if (strcmp(name, "org.kde.StatusNotifierWatcher") == 0 && new_owner &&
      new_owner[0] != '\0') {
    register_status_notifier_item();
  }

  return DBUS_HANDLER_RESULT_NOT_YET_HANDLED;
}

static void append_variant_basic(DBusMessageIter *parent, int type,
                                 const char *sig, const void *value) {
  DBusMessageIter variant_iter;
  dbus_message_iter_open_container(parent, DBUS_TYPE_VARIANT, sig,
                                   &variant_iter);
  dbus_message_iter_append_basic(&variant_iter, type, value);
  dbus_message_iter_close_container(parent, &variant_iter);
}

static void append_variant_string(DBusMessageIter *parent, const char *s) {
  append_variant_basic(parent, DBUS_TYPE_STRING, "s", &s);
}

static void append_variant_bool(DBusMessageIter *parent, dbus_bool_t b) {
  append_variant_basic(parent, DBUS_TYPE_BOOLEAN, "b", &b);
}

static void append_variant_int32(DBusMessageIter *parent, dbus_int32_t v) {
  append_variant_basic(parent, DBUS_TYPE_INT32, "i", &v);
}

static void append_variant_uint32(DBusMessageIter *parent, dbus_uint32_t v) {
  append_variant_basic(parent, DBUS_TYPE_UINT32, "u", &v);
}

static void append_variant_object_path(DBusMessageIter *parent,
                                       const char *path) {
  append_variant_basic(parent, DBUS_TYPE_OBJECT_PATH, "o", &path);
}

static void append_variant_string_array(DBusMessageIter *parent,
                                        const char *const *items, int count) {
  DBusMessageIter variant_iter;
  dbus_message_iter_open_container(parent, DBUS_TYPE_VARIANT, "as",
                                   &variant_iter);
  DBusMessageIter array_iter;
  dbus_message_iter_open_container(&variant_iter, DBUS_TYPE_ARRAY, "s",
                                   &array_iter);
  for (int i = 0; i < count; i++) {
    dbus_message_iter_append_basic(&array_iter, DBUS_TYPE_STRING, &items[i]);
  }
  dbus_message_iter_close_container(&variant_iter, &array_iter);
  dbus_message_iter_close_container(parent, &variant_iter);
}

static void append_icon_pixmap_variant_locked(DBusMessageIter *parent,
                                              bool with_data) {
  DBusMessageIter variant_iter;
  dbus_message_iter_open_container(parent, DBUS_TYPE_VARIANT, "a(iiay)",
                                   &variant_iter);
  DBusMessageIter array_iter;
  dbus_message_iter_open_container(&variant_iter, DBUS_TYPE_ARRAY, "(iiay)",
                                   &array_iter);

  if (with_data && g_state.icon_argb) {
    DBusMessageIter struct_iter;
    dbus_message_iter_open_container(&array_iter, DBUS_TYPE_STRUCT, NULL,
                                     &struct_iter);
    dbus_message_iter_append_basic(&struct_iter, DBUS_TYPE_INT32,
                                   &g_state.icon_width);
    dbus_message_iter_append_basic(&struct_iter, DBUS_TYPE_INT32,
                                   &g_state.icon_height);
    DBusMessageIter bytes_iter;
    dbus_message_iter_open_container(&struct_iter, DBUS_TYPE_ARRAY, "y",
                                     &bytes_iter);
    int n = (int)((size_t)g_state.icon_width * (size_t)g_state.icon_height * 4);
    if (n > 0) {
      const uint8_t *ptr = g_state.icon_argb;
      dbus_message_iter_append_fixed_array(&bytes_iter, DBUS_TYPE_BYTE, &ptr,
                                           n);
    }
    dbus_message_iter_close_container(&struct_iter, &bytes_iter);
    dbus_message_iter_close_container(&array_iter, &struct_iter);
  }

  dbus_message_iter_close_container(&variant_iter, &array_iter);
  dbus_message_iter_close_container(parent, &variant_iter);
}

static void append_empty_icon_pixmap_variant(DBusMessageIter *parent) {
  DBusMessageIter variant_iter;
  dbus_message_iter_open_container(parent, DBUS_TYPE_VARIANT, "a(iiay)",
                                   &variant_iter);
  DBusMessageIter array_iter;
  dbus_message_iter_open_container(&variant_iter, DBUS_TYPE_ARRAY, "(iiay)",
                                   &array_iter);
  dbus_message_iter_close_container(&variant_iter, &array_iter);
  dbus_message_iter_close_container(parent, &variant_iter);
}

static void append_tooltip_variant_locked(DBusMessageIter *parent) {
  DBusMessageIter variant_iter;
  dbus_message_iter_open_container(parent, DBUS_TYPE_VARIANT, "(sa(iiay)ss)",
                                   &variant_iter);
  DBusMessageIter struct_iter;
  dbus_message_iter_open_container(&variant_iter, DBUS_TYPE_STRUCT, NULL,
                                   &struct_iter);

  const char *empty = "";
  dbus_message_iter_append_basic(&struct_iter, DBUS_TYPE_STRING, &empty);

  DBusMessageIter icon_array_iter;
  dbus_message_iter_open_container(&struct_iter, DBUS_TYPE_ARRAY, "(iiay)",
                                   &icon_array_iter);
  dbus_message_iter_close_container(&struct_iter, &icon_array_iter);

  const char *title = g_state.tooltip;
  dbus_message_iter_append_basic(&struct_iter, DBUS_TYPE_STRING, &title);
  dbus_message_iter_append_basic(&struct_iter, DBUS_TYPE_STRING, &empty);

  dbus_message_iter_close_container(&variant_iter, &struct_iter);
  dbus_message_iter_close_container(parent, &variant_iter);
}

static bool sni_append_property(DBusMessageIter *parent, const char *name) {
  if (strcmp(name, "Category") == 0) {
    append_variant_string(parent, "ApplicationStatus");
  } else if (strcmp(name, "Id") == 0) {
    append_variant_string(parent, "wallpiper");
  } else if (strcmp(name, "Title") == 0) {
    append_variant_string(parent, "Wallpaper Engine");
  } else if (strcmp(name, "Status") == 0) {
    append_variant_string(parent, "Active");
  } else if (strcmp(name, "WindowId") == 0) {
    append_variant_int32(parent, 0);
  } else if (strcmp(name, "IconThemePath") == 0) {
    append_variant_string(parent, "");
  } else if (strcmp(name, "Menu") == 0) {
    append_variant_object_path(parent, MENU_PATH);
  } else if (strcmp(name, "ItemIsMenu") == 0) {
    append_variant_bool(parent, FALSE);
  } else if (strcmp(name, "IconName") == 0) {
    pthread_mutex_lock(&g_state.mutex);
    bool has_icon = g_state.icon_argb != NULL;
    pthread_mutex_unlock(&g_state.mutex);
    append_variant_string(parent,
                          has_icon ? "" : "preferences-desktop-wallpaper");
  } else if (strcmp(name, "IconPixmap") == 0) {
    pthread_mutex_lock(&g_state.mutex);
    append_icon_pixmap_variant_locked(parent, true);
    pthread_mutex_unlock(&g_state.mutex);
  } else if (strcmp(name, "OverlayIconName") == 0) {
    append_variant_string(parent, "");
  } else if (strcmp(name, "OverlayIconPixmap") == 0) {
    append_empty_icon_pixmap_variant(parent);
  } else if (strcmp(name, "AttentionIconName") == 0) {
    append_variant_string(parent, "");
  } else if (strcmp(name, "AttentionIconPixmap") == 0) {
    append_empty_icon_pixmap_variant(parent);
  } else if (strcmp(name, "AttentionMovieName") == 0) {
    append_variant_string(parent, "");
  } else if (strcmp(name, "ToolTip") == 0) {
    pthread_mutex_lock(&g_state.mutex);
    append_tooltip_variant_locked(parent);
    pthread_mutex_unlock(&g_state.mutex);
  } else {
    return false;
  }
  return true;
}

static const char *const SNI_PROPERTY_NAMES[] = {
    "Category",
    "Id",
    "Title",
    "Status",
    "WindowId",
    "IconThemePath",
    "Menu",
    "ItemIsMenu",
    "IconName",
    "IconPixmap",
    "OverlayIconName",
    "OverlayIconPixmap",
    "AttentionIconName",
    "AttentionIconPixmap",
    "AttentionMovieName",
    "ToolTip",
};
#define SNI_PROPERTY_COUNT                                                     \
  (sizeof(SNI_PROPERTY_NAMES) / sizeof(SNI_PROPERTY_NAMES[0]))

static bool menu_append_property(DBusMessageIter *parent, const char *name) {
  if (strcmp(name, "Version") == 0) {
    append_variant_uint32(parent, 3);
  } else if (strcmp(name, "TextDirection") == 0) {
    append_variant_string(parent, "ltr");
  } else if (strcmp(name, "Status") == 0) {
    append_variant_string(parent, "normal");
  } else if (strcmp(name, "IconThemePath") == 0) {
    append_variant_string_array(parent, NULL, 0);
  } else {
    return false;
  }
  return true;
}

static const char *const MENU_PROPERTY_NAMES[] = {
    "Version",
    "TextDirection",
    "Status",
    "IconThemePath",
};
#define MENU_PROPERTY_COUNT                                                    \
  (sizeof(MENU_PROPERTY_NAMES) / sizeof(MENU_PROPERTY_NAMES[0]))

static void send_reply(DBusConnection *conn, DBusMessage *reply) {
  if (reply) {
    dbus_connection_send(conn, reply, NULL);
    dbus_message_unref(reply);
  }
}

static void send_error(DBusConnection *conn, DBusMessage *msg, const char *name,
                       const char *message) {
  DBusMessage *err = dbus_message_new_error(msg, name, message);
  send_reply(conn, err);
}

static DBusHandlerResult handle_properties_get(DBusConnection *conn,
                                               DBusMessage *msg, bool is_sni) {
  const char *iface_arg = NULL;
  const char *prop_name = NULL;
  DBusError err;
  dbus_error_init(&err);
  if (!dbus_message_get_args(msg, &err, DBUS_TYPE_STRING, &iface_arg,
                             DBUS_TYPE_STRING, &prop_name, DBUS_TYPE_INVALID)) {
    dbus_error_free(&err);
    send_error(conn, msg, DBUS_ERROR_INVALID_ARGS, "expected (ss)");
    return DBUS_HANDLER_RESULT_HANDLED;
  }

  DBusMessage *reply = dbus_message_new_method_return(msg);
  DBusMessageIter iter;
  dbus_message_iter_init_append(reply, &iter);

  bool ok = is_sni ? sni_append_property(&iter, prop_name)
                   : menu_append_property(&iter, prop_name);
  if (!ok) {
    dbus_message_unref(reply);
    send_error(conn, msg, "org.freedesktop.DBus.Error.UnknownProperty",
               prop_name);
    return DBUS_HANDLER_RESULT_HANDLED;
  }

  send_reply(conn, reply);
  return DBUS_HANDLER_RESULT_HANDLED;
}

static DBusHandlerResult
handle_properties_get_all(DBusConnection *conn, DBusMessage *msg, bool is_sni) {
  DBusMessage *reply = dbus_message_new_method_return(msg);
  DBusMessageIter iter;
  dbus_message_iter_init_append(reply, &iter);

  DBusMessageIter dict_iter;
  dbus_message_iter_open_container(&iter, DBUS_TYPE_ARRAY, "{sv}", &dict_iter);

  size_t count = is_sni ? SNI_PROPERTY_COUNT : MENU_PROPERTY_COUNT;
  const char *const *names = is_sni ? SNI_PROPERTY_NAMES : MENU_PROPERTY_NAMES;
  for (size_t i = 0; i < count; i++) {
    DBusMessageIter entry_iter;
    dbus_message_iter_open_container(&dict_iter, DBUS_TYPE_DICT_ENTRY, NULL,
                                     &entry_iter);
    dbus_message_iter_append_basic(&entry_iter, DBUS_TYPE_STRING, &names[i]);
    if (is_sni) {
      sni_append_property(&entry_iter, names[i]);
    } else {
      menu_append_property(&entry_iter, names[i]);
    }
    dbus_message_iter_close_container(&dict_iter, &entry_iter);
  }

  dbus_message_iter_close_container(&iter, &dict_iter);
  send_reply(conn, reply);
  return DBUS_HANDLER_RESULT_HANDLED;
}

static DBusHandlerResult handle_sni_method(DBusConnection *conn,
                                           DBusMessage *msg) {
  const char *member = dbus_message_get_member(msg);

  if (strcmp(member, "Activate") == 0 ||
      strcmp(member, "SecondaryActivate") == 0 ||
      strcmp(member, "ContextMenu") == 0 || strcmp(member, "Scroll") == 0) {
    if (strcmp(member, "Activate") == 0) {
      wp_tray_write_click(0);
    } else if (strcmp(member, "SecondaryActivate") == 0) {
      wp_tray_write_click(1);
    }
    DBusMessage *reply = dbus_message_new_method_return(msg);
    send_reply(conn, reply);
    return DBUS_HANDLER_RESULT_HANDLED;
  }

  return DBUS_HANDLER_RESULT_NOT_YET_HANDLED;
}

typedef struct {
  int32_t id;
  char label[256];
  bool has_label;
  bool enabled_present;
  dbus_bool_t enabled_value;
  bool is_separator;
  bool toggle_present;
  const char *toggle_type;
  dbus_int32_t toggle_state;
  bool children_display_present;
  bool has_children;
} menu_node_props_t;

static void compute_node_properties(const wp_tray_entry_t *entry,
                                    bool has_children, menu_node_props_t *out) {
  memset(out, 0, sizeof(*out));
  if (!entry) {
    return;
  }

  out->id = (int32_t)entry->id;
  out->has_children = has_children;

  if (entry->type_flags & WP_TRAY_MFT_SEPARATOR) {
    out->is_separator = true;
    return;
  }

  snprintf(out->label, sizeof(out->label), "%s", entry->text);
  out->has_label = out->label[0] != '\0';

  bool enabled = (entry->state & WP_TRAY_MFS_DISABLED) == 0;
  if (!enabled) {
    out->enabled_present = true;
    out->enabled_value = FALSE;
  }

  if (!has_children && (entry->state & WP_TRAY_MFS_CHECKED) != 0) {
    out->toggle_present = true;
    out->toggle_type = "checkmark";
    out->toggle_state = 1;
  }

  if (has_children) {
    out->children_display_present = true;
  }
}

static void append_node_properties(DBusMessageIter *struct_iter,
                                   const menu_node_props_t *props) {
  DBusMessageIter dict_iter;
  dbus_message_iter_open_container(struct_iter, DBUS_TYPE_ARRAY, "{sv}",
                                   &dict_iter);

  if (props->is_separator) {
    DBusMessageIter entry_iter;
    dbus_message_iter_open_container(&dict_iter, DBUS_TYPE_DICT_ENTRY, NULL,
                                     &entry_iter);
    const char *key = "type";
    dbus_message_iter_append_basic(&entry_iter, DBUS_TYPE_STRING, &key);
    append_variant_string(&entry_iter, "separator");
    dbus_message_iter_close_container(&dict_iter, &entry_iter);
  } else {
    if (props->has_label) {
      DBusMessageIter entry_iter;
      dbus_message_iter_open_container(&dict_iter, DBUS_TYPE_DICT_ENTRY, NULL,
                                       &entry_iter);
      const char *key = "label";
      dbus_message_iter_append_basic(&entry_iter, DBUS_TYPE_STRING, &key);
      append_variant_string(&entry_iter, props->label);
      dbus_message_iter_close_container(&dict_iter, &entry_iter);
    }
    if (props->enabled_present) {
      DBusMessageIter entry_iter;
      dbus_message_iter_open_container(&dict_iter, DBUS_TYPE_DICT_ENTRY, NULL,
                                       &entry_iter);
      const char *key = "enabled";
      dbus_message_iter_append_basic(&entry_iter, DBUS_TYPE_STRING, &key);
      append_variant_bool(&entry_iter, props->enabled_value);
      dbus_message_iter_close_container(&dict_iter, &entry_iter);
    }
    if (props->toggle_present) {
      DBusMessageIter entry_iter;
      dbus_message_iter_open_container(&dict_iter, DBUS_TYPE_DICT_ENTRY, NULL,
                                       &entry_iter);
      const char *key = "toggle-type";
      dbus_message_iter_append_basic(&entry_iter, DBUS_TYPE_STRING, &key);
      append_variant_string(&entry_iter, props->toggle_type);
      dbus_message_iter_close_container(&dict_iter, &entry_iter);

      dbus_message_iter_open_container(&dict_iter, DBUS_TYPE_DICT_ENTRY, NULL,
                                       &entry_iter);
      key = "toggle-state";
      dbus_message_iter_append_basic(&entry_iter, DBUS_TYPE_STRING, &key);
      append_variant_int32(&entry_iter, props->toggle_state);
      dbus_message_iter_close_container(&dict_iter, &entry_iter);
    }
    if (props->children_display_present) {
      DBusMessageIter entry_iter;
      dbus_message_iter_open_container(&dict_iter, DBUS_TYPE_DICT_ENTRY, NULL,
                                       &entry_iter);
      const char *key = "children-display";
      dbus_message_iter_append_basic(&entry_iter, DBUS_TYPE_STRING, &key);
      append_variant_string(&entry_iter, "submenu");
      dbus_message_iter_close_container(&dict_iter, &entry_iter);
    }
  }

  dbus_message_iter_close_container(struct_iter, &dict_iter);
}

static int32_t entry_display_id(int index) { return (int32_t)index + 1; }

static int find_index_by_id(const wp_tray_entries_t *entries, int32_t id) {
  int index = id - 1;
  if (index < 0 || index >= (int)entries->count) {
    return -1;
  }
  return index;
}

static int children_run_length(const wp_tray_entries_t *entries,
                               int start_index, int32_t child_depth) {
  int n = 0;
  for (int i = start_index; i < (int)entries->count; i++) {
    if (entries->entries[i].depth != child_depth) {
      break;
    }
    n++;
  }
  return n;
}

static void build_menu_node(DBusMessageIter *parent_iter,
                            const wp_tray_entries_t *entries,
                            const wp_tray_entry_t *entry, int32_t id,
                            int start_index, int32_t child_depth,
                            int recursion_remaining) {
  DBusMessageIter struct_iter;
  dbus_message_iter_open_container(parent_iter, DBUS_TYPE_STRUCT, NULL,
                                   &struct_iter);
  dbus_message_iter_append_basic(&struct_iter, DBUS_TYPE_INT32, &id);

  bool has_children = start_index >= 0 && start_index < (int)entries->count &&
                      entries->entries[start_index].depth == child_depth;

  menu_node_props_t props;
  compute_node_properties(entry, has_children, &props);
  append_node_properties(&struct_iter, &props);

  DBusMessageIter children_iter;
  dbus_message_iter_open_container(&struct_iter, DBUS_TYPE_ARRAY, "v",
                                   &children_iter);

  if (has_children && recursion_remaining != 0) {
    int next_remaining =
        recursion_remaining > 0 ? recursion_remaining - 1 : recursion_remaining;
    int i = start_index;
    while (i < (int)entries->count &&
           entries->entries[i].depth == child_depth) {
      const wp_tray_entry_t *child = &entries->entries[i];
      DBusMessageIter variant_iter;
      dbus_message_iter_open_container(&children_iter, DBUS_TYPE_VARIANT,
                                       "(ia{sv}av)", &variant_iter);
      build_menu_node(&variant_iter, entries, child, entry_display_id(i), i + 1,
                      child_depth + 1, next_remaining);
      dbus_message_iter_close_container(&children_iter, &variant_iter);

      int run = children_run_length(entries, i + 1, child_depth + 1);
      i = i + 1 + run;
    }
  }

  dbus_message_iter_close_container(&struct_iter, &children_iter);
  dbus_message_iter_close_container(parent_iter, &struct_iter);
}

static DBusHandlerResult handle_menu_get_layout(DBusConnection *conn,
                                                DBusMessage *msg) {
  dbus_int32_t parent_id = 0;
  dbus_int32_t recursion_depth = -1;
  DBusMessageIter arg_iter;
  dbus_message_iter_init(msg, &arg_iter);
  dbus_message_iter_get_basic(&arg_iter, &parent_id);
  dbus_message_iter_next(&arg_iter);
  dbus_message_iter_get_basic(&arg_iter, &recursion_depth);

  pthread_mutex_lock(&g_state.mutex);
  wp_tray_entries_t entries = g_state.entries;
  uint32_t revision = g_state.revision;
  pthread_mutex_unlock(&g_state.mutex);

  int start_index;
  int32_t child_depth;
  const wp_tray_entry_t *entry_ptr = NULL;

  if (parent_id == 0) {
    start_index = 0;
    child_depth = 0;
  } else {
    int idx = find_index_by_id(&entries, parent_id);
    if (idx < 0) {
      send_error(conn, msg, "org.freedesktop.DBus.Error.Failed",
                 "parentId not found");
      return DBUS_HANDLER_RESULT_HANDLED;
    }
    entry_ptr = &entries.entries[idx];
    start_index = idx + 1;
    child_depth = entry_ptr->depth + 1;
  }

  DBusMessage *reply = dbus_message_new_method_return(msg);
  DBusMessageIter iter;
  dbus_message_iter_init_append(reply, &iter);
  dbus_message_iter_append_basic(&iter, DBUS_TYPE_UINT32, &revision);

  build_menu_node(&iter, &entries, entry_ptr, parent_id, start_index,
                  child_depth, recursion_depth);

  send_reply(conn, reply);
  return DBUS_HANDLER_RESULT_HANDLED;
}

static DBusHandlerResult handle_menu_get_group_properties(DBusConnection *conn,
                                                          DBusMessage *msg) {
  DBusMessageIter arg_iter;
  dbus_message_iter_init(msg, &arg_iter);

  DBusMessageIter ids_iter;
  dbus_message_iter_recurse(&arg_iter, &ids_iter);

  pthread_mutex_lock(&g_state.mutex);
  wp_tray_entries_t entries = g_state.entries;
  pthread_mutex_unlock(&g_state.mutex);

  DBusMessage *reply = dbus_message_new_method_return(msg);
  DBusMessageIter iter;
  dbus_message_iter_init_append(reply, &iter);
  DBusMessageIter array_iter;
  dbus_message_iter_open_container(&iter, DBUS_TYPE_ARRAY, "(ia{sv})",
                                   &array_iter);

  while (dbus_message_iter_get_arg_type(&ids_iter) == DBUS_TYPE_INT32) {
    dbus_int32_t id;
    dbus_message_iter_get_basic(&ids_iter, &id);
    dbus_message_iter_next(&ids_iter);

    int idx = find_index_by_id(&entries, id);
    if (idx < 0) {
      continue;
    }

    int child_depth = entries.entries[idx].depth + 1;
    bool has_children = (idx + 1) < (int)entries.count &&
                        entries.entries[idx + 1].depth == child_depth;

    menu_node_props_t props;
    compute_node_properties(&entries.entries[idx], has_children, &props);

    DBusMessageIter struct_iter;
    dbus_message_iter_open_container(&array_iter, DBUS_TYPE_STRUCT, NULL,
                                     &struct_iter);
    dbus_message_iter_append_basic(&struct_iter, DBUS_TYPE_INT32, &id);
    append_node_properties(&struct_iter, &props);
    dbus_message_iter_close_container(&array_iter, &struct_iter);
  }

  dbus_message_iter_close_container(&iter, &array_iter);
  send_reply(conn, reply);
  return DBUS_HANDLER_RESULT_HANDLED;
}

static DBusHandlerResult handle_menu_event(DBusConnection *conn,
                                           DBusMessage *msg) {
  dbus_int32_t id = 0;
  const char *event_id = "";
  DBusMessageIter arg_iter;
  dbus_message_iter_init(msg, &arg_iter);
  dbus_message_iter_get_basic(&arg_iter, &id);
  dbus_message_iter_next(&arg_iter);
  dbus_message_iter_get_basic(&arg_iter, &event_id);

  if (strcmp(event_id, "clicked") == 0 && id != 0) {
    pthread_mutex_lock(&g_state.mutex);
    wp_tray_entries_t entries = g_state.entries;
    pthread_mutex_unlock(&g_state.mutex);

    int idx = find_index_by_id(&entries, id);
    if (idx >= 0 &&
        !(entries.entries[idx].type_flags & WP_TRAY_MFT_SEPARATOR)) {
      wp_tray_debug_log("event: clicked dbus_id=%d -> win32_id=%u", id,
                        entries.entries[idx].id);
      wp_tray_send_menu_command(entries.entries[idx].id);
    } else {
      wp_tray_debug_log("event: clicked dbus_id=%d did not resolve to an "
                        "entry (idx=%d)",
                        id, idx);
    }
  }

  DBusMessage *reply = dbus_message_new_method_return(msg);
  send_reply(conn, reply);
  return DBUS_HANDLER_RESULT_HANDLED;
}

static DBusHandlerResult handle_menu_about_to_show(DBusConnection *conn,
                                                   DBusMessage *msg) {
  bool changed = refresh_menu_from_win32();

  DBusMessage *reply = dbus_message_new_method_return(msg);
  DBusMessageIter iter;
  dbus_message_iter_init_append(reply, &iter);
  dbus_bool_t need_update = changed ? TRUE : FALSE;
  dbus_message_iter_append_basic(&iter, DBUS_TYPE_BOOLEAN, &need_update);
  send_reply(conn, reply);
  return DBUS_HANDLER_RESULT_HANDLED;
}

static DBusHandlerResult handle_menu_about_to_show_group(DBusConnection *conn,
                                                         DBusMessage *msg) {
  DBusMessageIter arg_iter;
  dbus_message_iter_init(msg, &arg_iter);
  DBusMessageIter ids_iter;
  dbus_message_iter_recurse(&arg_iter, &ids_iter);

  dbus_int32_t ids[WP_TRAY_MAX_ENTRIES];
  size_t id_count = 0;
  while (dbus_message_iter_get_arg_type(&ids_iter) == DBUS_TYPE_INT32 &&
         id_count < WP_TRAY_MAX_ENTRIES) {
    dbus_message_iter_get_basic(&ids_iter, &ids[id_count]);
    id_count++;
    dbus_message_iter_next(&ids_iter);
  }

  bool changed = refresh_menu_from_win32();

  DBusMessage *reply = dbus_message_new_method_return(msg);
  DBusMessageIter iter;
  dbus_message_iter_init_append(reply, &iter);

  DBusMessageIter updates_iter;
  dbus_message_iter_open_container(&iter, DBUS_TYPE_ARRAY, "i", &updates_iter);
  if (changed) {
    for (size_t i = 0; i < id_count; i++) {
      dbus_message_iter_append_basic(&updates_iter, DBUS_TYPE_INT32, &ids[i]);
    }
  }
  dbus_message_iter_close_container(&iter, &updates_iter);

  DBusMessageIter errors_iter;
  dbus_message_iter_open_container(&iter, DBUS_TYPE_ARRAY, "i", &errors_iter);
  dbus_message_iter_close_container(&iter, &errors_iter);

  send_reply(conn, reply);
  return DBUS_HANDLER_RESULT_HANDLED;
}

static DBusHandlerResult handle_menu_event_group(DBusConnection *conn,
                                                 DBusMessage *msg) {
  pthread_mutex_lock(&g_state.mutex);
  wp_tray_entries_t entries = g_state.entries;
  pthread_mutex_unlock(&g_state.mutex);

  DBusMessageIter arg_iter;
  dbus_message_iter_init(msg, &arg_iter);
  DBusMessageIter events_iter;
  dbus_message_iter_recurse(&arg_iter, &events_iter);

  dbus_int32_t error_ids[WP_TRAY_MAX_ENTRIES];
  size_t error_count = 0;

  while (dbus_message_iter_get_arg_type(&events_iter) == DBUS_TYPE_STRUCT) {
    DBusMessageIter event_struct;
    dbus_message_iter_recurse(&events_iter, &event_struct);

    dbus_int32_t id = 0;
    dbus_message_iter_get_basic(&event_struct, &id);
    dbus_message_iter_next(&event_struct);

    const char *event_id = "";
    dbus_message_iter_get_basic(&event_struct, &event_id);

    int idx = find_index_by_id(&entries, id);
    bool ok =
        idx >= 0 && !(entries.entries[idx].type_flags & WP_TRAY_MFT_SEPARATOR);

    if (strcmp(event_id, "clicked") == 0 && ok) {
      wp_tray_debug_log("event_group: clicked dbus_id=%d -> win32_id=%u", id,
                        entries.entries[idx].id);
      wp_tray_send_menu_command(entries.entries[idx].id);
    } else if (!ok && error_count < WP_TRAY_MAX_ENTRIES) {
      error_ids[error_count++] = id;
    }

    dbus_message_iter_next(&events_iter);
  }

  DBusMessage *reply = dbus_message_new_method_return(msg);
  DBusMessageIter iter;
  dbus_message_iter_init_append(reply, &iter);
  DBusMessageIter errors_iter;
  dbus_message_iter_open_container(&iter, DBUS_TYPE_ARRAY, "i", &errors_iter);
  for (size_t i = 0; i < error_count; i++) {
    dbus_message_iter_append_basic(&errors_iter, DBUS_TYPE_INT32,
                                   &error_ids[i]);
  }
  dbus_message_iter_close_container(&iter, &errors_iter);
  send_reply(conn, reply);
  return DBUS_HANDLER_RESULT_HANDLED;
}

static DBusHandlerResult handle_menu_get_property(DBusConnection *conn,
                                                  DBusMessage *msg) {
  DBusMessage *reply = dbus_message_new_method_return(msg);
  DBusMessageIter iter;
  dbus_message_iter_init_append(reply, &iter);
  append_variant_bool(&iter, FALSE);
  send_reply(conn, reply);
  return DBUS_HANDLER_RESULT_HANDLED;
}

static DBusHandlerResult handle_menu_method(DBusConnection *conn,
                                            DBusMessage *msg) {
  const char *member = dbus_message_get_member(msg);

  if (strcmp(member, "GetLayout") == 0) {
    return handle_menu_get_layout(conn, msg);
  }
  if (strcmp(member, "GetGroupProperties") == 0) {
    return handle_menu_get_group_properties(conn, msg);
  }
  if (strcmp(member, "Event") == 0) {
    return handle_menu_event(conn, msg);
  }
  if (strcmp(member, "EventGroup") == 0) {
    return handle_menu_event_group(conn, msg);
  }
  if (strcmp(member, "AboutToShow") == 0) {
    return handle_menu_about_to_show(conn, msg);
  }
  if (strcmp(member, "AboutToShowGroup") == 0) {
    return handle_menu_about_to_show_group(conn, msg);
  }
  if (strcmp(member, "GetProperty") == 0) {
    return handle_menu_get_property(conn, msg);
  }

  return DBUS_HANDLER_RESULT_NOT_YET_HANDLED;
}

static const char *const SNI_INTROSPECT_XML =
    "<!DOCTYPE node PUBLIC \"-//freedesktop//DTD D-BUS Object Introspection "
    "1.0//EN\"\n"
    "\"http://www.freedesktop.org/standards/dbus/1.0/introspect.dtd\">\n"
    "<node>\n"
    "  <interface name=\"org.freedesktop.DBus.Properties\">\n"
    "    <method name=\"Get\"><arg type=\"s\" direction=\"in\"/><arg "
    "type=\"s\" direction=\"in\"/>"
    "<arg type=\"v\" direction=\"out\"/></method>\n"
    "    <method name=\"GetAll\"><arg type=\"s\" direction=\"in\"/><arg "
    "type=\"a{sv}\" direction=\"out\"/></method>\n"
    "  </interface>\n"
    "  <interface name=\"org.kde.StatusNotifierItem\">\n"
    "    <property name=\"Category\" type=\"s\" access=\"read\"/>\n"
    "    <property name=\"Id\" type=\"s\" access=\"read\"/>\n"
    "    <property name=\"Title\" type=\"s\" access=\"read\"/>\n"
    "    <property name=\"Status\" type=\"s\" access=\"read\"/>\n"
    "    <property name=\"WindowId\" type=\"i\" access=\"read\"/>\n"
    "    <property name=\"IconThemePath\" type=\"s\" access=\"read\"/>\n"
    "    <property name=\"Menu\" type=\"o\" access=\"read\"/>\n"
    "    <property name=\"ItemIsMenu\" type=\"b\" access=\"read\"/>\n"
    "    <property name=\"IconName\" type=\"s\" access=\"read\"/>\n"
    "    <property name=\"IconPixmap\" type=\"a(iiay)\" access=\"read\"/>\n"
    "    <property name=\"OverlayIconName\" type=\"s\" access=\"read\"/>\n"
    "    <property name=\"OverlayIconPixmap\" type=\"a(iiay)\" "
    "access=\"read\"/>\n"
    "    <property name=\"AttentionIconName\" type=\"s\" access=\"read\"/>\n"
    "    <property name=\"AttentionIconPixmap\" type=\"a(iiay)\" "
    "access=\"read\"/>\n"
    "    <property name=\"AttentionMovieName\" type=\"s\" access=\"read\"/>\n"
    "    <property name=\"ToolTip\" type=\"(sa(iiay)ss)\" access=\"read\"/>\n"
    "    <method name=\"ContextMenu\"><arg name=\"x\" type=\"i\" "
    "direction=\"in\"/>"
    "<arg name=\"y\" type=\"i\" direction=\"in\"/></method>\n"
    "    <method name=\"Activate\"><arg name=\"x\" type=\"i\" "
    "direction=\"in\"/>"
    "<arg name=\"y\" type=\"i\" direction=\"in\"/></method>\n"
    "    <method name=\"SecondaryActivate\"><arg name=\"x\" type=\"i\" "
    "direction=\"in\"/>"
    "<arg name=\"y\" type=\"i\" direction=\"in\"/></method>\n"
    "    <method name=\"Scroll\"><arg name=\"delta\" type=\"i\" "
    "direction=\"in\"/>"
    "<arg name=\"orientation\" type=\"s\" direction=\"in\"/></method>\n"
    "    <signal name=\"NewTitle\"/>\n"
    "    <signal name=\"NewIcon\"/>\n"
    "    <signal name=\"NewAttentionIcon\"/>\n"
    "    <signal name=\"NewOverlayIcon\"/>\n"
    "    <signal name=\"NewToolTip\"/>\n"
    "    <signal name=\"NewStatus\"><arg name=\"status\" "
    "type=\"s\"/></signal>\n"
    "  </interface>\n"
    "</node>\n";

static const char *const MENU_INTROSPECT_XML =
    "<!DOCTYPE node PUBLIC \"-//freedesktop//DTD D-BUS Object Introspection "
    "1.0//EN\"\n"
    "\"http://www.freedesktop.org/standards/dbus/1.0/introspect.dtd\">\n"
    "<node>\n"
    "  <interface name=\"org.freedesktop.DBus.Properties\">\n"
    "    <method name=\"Get\"><arg type=\"s\" direction=\"in\"/><arg "
    "type=\"s\" direction=\"in\"/>"
    "<arg type=\"v\" direction=\"out\"/></method>\n"
    "    <method name=\"GetAll\"><arg type=\"s\" direction=\"in\"/><arg "
    "type=\"a{sv}\" direction=\"out\"/></method>\n"
    "  </interface>\n"
    "  <interface name=\"com.canonical.dbusmenu\">\n"
    "    <property name=\"Version\" type=\"u\" access=\"read\"/>\n"
    "    <property name=\"TextDirection\" type=\"s\" access=\"read\"/>\n"
    "    <property name=\"Status\" type=\"s\" access=\"read\"/>\n"
    "    <property name=\"IconThemePath\" type=\"as\" access=\"read\"/>\n"
    "    <method name=\"GetLayout\">\n"
    "      <arg type=\"i\" name=\"parentId\" direction=\"in\"/>\n"
    "      <arg type=\"i\" name=\"recursionDepth\" direction=\"in\"/>\n"
    "      <arg type=\"as\" name=\"propertyNames\" direction=\"in\"/>\n"
    "      <arg type=\"u\" name=\"revision\" direction=\"out\"/>\n"
    "      <arg type=\"(ia{sv}av)\" name=\"layout\" direction=\"out\"/>\n"
    "    </method>\n"
    "    <method name=\"GetGroupProperties\">\n"
    "      <arg type=\"ai\" name=\"ids\" direction=\"in\"/>\n"
    "      <arg type=\"as\" name=\"propertyNames\" direction=\"in\"/>\n"
    "      <arg type=\"a(ia{sv})\" name=\"properties\" direction=\"out\"/>\n"
    "    </method>\n"
    "    <method name=\"GetProperty\">\n"
    "      <arg type=\"i\" name=\"id\" direction=\"in\"/>\n"
    "      <arg type=\"s\" name=\"name\" direction=\"in\"/>\n"
    "      <arg type=\"v\" name=\"value\" direction=\"out\"/>\n"
    "    </method>\n"
    "    <method name=\"Event\">\n"
    "      <arg type=\"i\" name=\"id\" direction=\"in\"/>\n"
    "      <arg type=\"s\" name=\"eventId\" direction=\"in\"/>\n"
    "      <arg type=\"v\" name=\"data\" direction=\"in\"/>\n"
    "      <arg type=\"u\" name=\"timestamp\" direction=\"in\"/>\n"
    "    </method>\n"
    "    <method name=\"EventGroup\">\n"
    "      <arg type=\"a(isvu)\" name=\"events\" direction=\"in\"/>\n"
    "      <arg type=\"ai\" name=\"idErrors\" direction=\"out\"/>\n"
    "    </method>\n"
    "    <method name=\"AboutToShow\">\n"
    "      <arg type=\"i\" name=\"id\" direction=\"in\"/>\n"
    "      <arg type=\"b\" name=\"needUpdate\" direction=\"out\"/>\n"
    "    </method>\n"
    "    <method name=\"AboutToShowGroup\">\n"
    "      <arg type=\"ai\" name=\"ids\" direction=\"in\"/>\n"
    "      <arg type=\"ai\" name=\"updatesNeeded\" direction=\"out\"/>\n"
    "      <arg type=\"ai\" name=\"idErrors\" direction=\"out\"/>\n"
    "    </method>\n"
    "    <signal name=\"ItemsPropertiesUpdated\">\n"
    "      <arg type=\"a(ia{sv})\"/><arg type=\"a(ias)\"/>\n"
    "    </signal>\n"
    "    <signal name=\"LayoutUpdated\">\n"
    "      <arg type=\"u\" name=\"revision\"/><arg type=\"i\" "
    "name=\"parent\"/>\n"
    "    </signal>\n"
    "  </interface>\n"
    "</node>\n";

static const char *const ROOT_INTROSPECT_XML =
    "<!DOCTYPE node PUBLIC \"-//freedesktop//DTD D-BUS Object Introspection "
    "1.0//EN\"\n"
    "\"http://www.freedesktop.org/standards/dbus/1.0/introspect.dtd\">\n"
    "<node>\n"
    "  <node name=\"StatusNotifierItem\"/>\n"
    "  <node name=\"MenuBar\"/>\n"
    "</node>\n";

static DBusHandlerResult handle_introspect(DBusConnection *conn,
                                           DBusMessage *msg, const char *xml) {
  DBusMessage *reply = dbus_message_new_method_return(msg);
  dbus_message_append_args(reply, DBUS_TYPE_STRING, &xml, DBUS_TYPE_INVALID);
  send_reply(conn, reply);
  return DBUS_HANDLER_RESULT_HANDLED;
}

static DBusHandlerResult
sni_message_handler(DBusConnection *conn, DBusMessage *msg, void *user_data) {
  (void)user_data;
  if (dbus_message_is_method_call(msg, PROPS_IFACE, "Get")) {
    return handle_properties_get(conn, msg, true);
  }
  if (dbus_message_is_method_call(msg, PROPS_IFACE, "GetAll")) {
    return handle_properties_get_all(conn, msg, true);
  }
  if (dbus_message_is_method_call(msg, INTROSPECT_IFACE, "Introspect")) {
    return handle_introspect(conn, msg, SNI_INTROSPECT_XML);
  }
  if (dbus_message_has_interface(msg, SNI_IFACE) ||
      dbus_message_get_interface(msg) == NULL) {
    return handle_sni_method(conn, msg);
  }
  return DBUS_HANDLER_RESULT_NOT_YET_HANDLED;
}

static DBusHandlerResult
menu_message_handler(DBusConnection *conn, DBusMessage *msg, void *user_data) {
  (void)user_data;
  if (dbus_message_is_method_call(msg, PROPS_IFACE, "Get")) {
    return handle_properties_get(conn, msg, false);
  }
  if (dbus_message_is_method_call(msg, PROPS_IFACE, "GetAll")) {
    return handle_properties_get_all(conn, msg, false);
  }
  if (dbus_message_is_method_call(msg, INTROSPECT_IFACE, "Introspect")) {
    return handle_introspect(conn, msg, MENU_INTROSPECT_XML);
  }
  if (dbus_message_has_interface(msg, MENU_IFACE) ||
      dbus_message_get_interface(msg) == NULL) {
    return handle_menu_method(conn, msg);
  }
  return DBUS_HANDLER_RESULT_NOT_YET_HANDLED;
}

static DBusHandlerResult
root_message_handler(DBusConnection *conn, DBusMessage *msg, void *user_data) {
  (void)user_data;
  if (dbus_message_is_method_call(msg, INTROSPECT_IFACE, "Introspect")) {
    return handle_introspect(conn, msg, ROOT_INTROSPECT_XML);
  }
  return DBUS_HANDLER_RESULT_NOT_YET_HANDLED;
}

static void emit_no_arg_signal(const char *path, const char *iface,
                               const char *signal_name) {
  DBusMessage *msg = dbus_message_new_signal(path, iface, signal_name);
  if (!msg) {
    return;
  }
  dbus_connection_send(g_state.conn, msg, NULL);
  dbus_message_unref(msg);
}

static void emit_layout_updated(uint32_t revision) {
  DBusMessage *msg =
      dbus_message_new_signal(MENU_PATH, MENU_IFACE, "LayoutUpdated");
  if (!msg) {
    return;
  }
  dbus_int32_t parent = 0;
  dbus_message_append_args(msg, DBUS_TYPE_UINT32, &revision, DBUS_TYPE_INT32,
                           &parent, DBUS_TYPE_INVALID);
  dbus_connection_send(g_state.conn, msg, NULL);
  dbus_message_unref(msg);
}

void wp_tray_state_on_menu_dump_changed(const wp_tray_entries_t *entries) {
  pthread_mutex_lock(&g_state.mutex);
  g_state.entries = *entries;
  g_state.generation++;
  pthread_cond_broadcast(&g_state.entries_cond);
  pthread_mutex_unlock(&g_state.mutex);
}

static bool timespec_ge(struct timespec a, struct timespec b) {
  if (a.tv_sec != b.tv_sec) {
    return a.tv_sec > b.tv_sec;
  }
  return a.tv_nsec >= b.tv_nsec;
}

static struct timespec timespec_add_ms(struct timespec t, long ms) {
  t.tv_nsec += ms * 1000000L;
  while (t.tv_nsec >= 1000000000L) {
    t.tv_sec += 1;
    t.tv_nsec -= 1000000000L;
  }
  return t;
}

static bool refresh_menu_from_win32(void) {
  pthread_mutex_lock(&g_state.mutex);
  if (g_state.refresh_in_progress) {
    wp_tray_debug_log("menu(): refresh already in flight, piggybacking");
    while (g_state.refresh_in_progress) {
      pthread_cond_wait(&g_state.entries_cond, &g_state.mutex);
    }
    size_t piggyback_count = g_state.entries.count;
    pthread_mutex_unlock(&g_state.mutex);
    wp_tray_debug_log("menu(): piggyback finished entries=%zu",
                      piggyback_count);
    return piggyback_count > 0;
  }
  g_state.refresh_in_progress = true;
  uint64_t start_generation = g_state.generation;
  pthread_mutex_unlock(&g_state.mutex);

  wp_tray_debug_log("menu(): called");
  wp_tray_debug_log("menu(): waiting for a fresh pull (start_generation=%llu",
                    (unsigned long long)start_generation);

  wp_tray_write_click(1);

  struct timespec deadline;
  clock_gettime(CLOCK_REALTIME, &deadline);
  deadline.tv_sec += MENU_REFRESH_TIMEOUT_SEC;

  pthread_mutex_lock(&g_state.mutex);
  bool timed_out = false;
  while (g_state.generation == start_generation) {
    struct timespec retry_deadline;
    clock_gettime(CLOCK_REALTIME, &retry_deadline);
    retry_deadline.tv_sec += MENU_REFRESH_RETRY_INTERVAL_SEC;
    bool final_wait = timespec_ge(retry_deadline, deadline);
    if (final_wait) {
      retry_deadline = deadline;
    }

    int rc = pthread_cond_timedwait(&g_state.entries_cond, &g_state.mutex,
                                    &retry_deadline);
    if (g_state.generation != start_generation) {
      break;
    }
    if (final_wait) {
      timed_out = true;
      break;
    }
    if (rc != 0) {
      pthread_mutex_unlock(&g_state.mutex);
      wp_tray_debug_log("menu(): retry wait elapsed, re-sending click");
      wp_tray_write_click(1);
      pthread_mutex_lock(&g_state.mutex);
    }
  }

  if (!timed_out) {
    uint64_t settled_generation = g_state.generation;
    struct timespec quiet_deadline;
    clock_gettime(CLOCK_REALTIME, &quiet_deadline);
    quiet_deadline = timespec_add_ms(quiet_deadline, MENU_REFRESH_SETTLE_MS);

    for (;;) {
      int rc = pthread_cond_timedwait(&g_state.entries_cond, &g_state.mutex,
                                      &quiet_deadline);
      if (g_state.generation != settled_generation) {
        settled_generation = g_state.generation;
        clock_gettime(CLOCK_REALTIME, &quiet_deadline);
        quiet_deadline =
            timespec_add_ms(quiet_deadline, MENU_REFRESH_SETTLE_MS);
        continue;
      }
      if (rc != 0) {
        break;
      }
    }
    wp_tray_debug_log("menu(): settled at generation=%llu",
                      (unsigned long long)settled_generation);
  }

  bool fetched = !timed_out;
  size_t entry_count = g_state.entries.count;
  if (fetched) {
    g_state.revision++;
  }
  uint32_t revision = g_state.revision;
  g_state.refresh_in_progress = false;
  pthread_cond_broadcast(&g_state.entries_cond);
  pthread_mutex_unlock(&g_state.mutex);

  wp_tray_debug_log("menu(): wait finished, timed_out=%d entries=%zu",
                    timed_out ? 1 : 0, entry_count);

  if (fetched) {
    emit_layout_updated(revision);
  }
  return fetched;
}

void wp_tray_state_on_icon_changed(wp_tray_icon_t *icon) {
  pthread_mutex_lock(&g_state.mutex);
  free(g_state.icon_argb);
  g_state.icon_width = icon->width;
  g_state.icon_height = icon->height;
  g_state.icon_argb = icon->pixels_argb;
  snprintf(g_state.tooltip, sizeof(g_state.tooltip), "%s", icon->tooltip);
  pthread_mutex_unlock(&g_state.mutex);

  icon->pixels_argb = NULL;

  emit_no_arg_signal(SNI_PATH, SNI_IFACE, "NewIcon");
  emit_no_arg_signal(SNI_PATH, SNI_IFACE, "NewToolTip");

  refresh_menu_from_win32();
}

static void *dbus_thread_main(void *arg) {
  (void)arg;
  while (dbus_connection_read_write_dispatch(g_state.conn, -1)) {
  }
  return NULL;
}

static void *tray_announce_thread_main(void *arg) {
  (void)arg;

  wp_tray_debug_log("spawn: pre-warming menu before announcing to the host");
  time_t prewarm_deadline = time(NULL) + WP_TRAY_PREWARM_MAX_SEC;
  while (!refresh_menu_from_win32() && time(NULL) < prewarm_deadline) {
    wp_tray_debug_log("spawn: pre-warm attempt failed, retrying");
  }

  pthread_t dbus_thread;
  if (pthread_create(&dbus_thread, NULL, dbus_thread_main, NULL) == 0) {
    pthread_detach(dbus_thread);
  }

  register_status_notifier_item();
  return NULL;
}

void wp_tray_spawn(void) {
  pthread_mutex_init(&g_state.mutex, NULL);
  pthread_cond_init(&g_state.entries_cond, NULL);
  g_state.icon_argb = NULL;
  g_state.entries.count = 0;
  g_state.generation = 0;
  g_state.revision = 0;
  g_state.refresh_in_progress = false;

  dbus_threads_init_default();

  DBusError err;
  dbus_error_init(&err);
  g_state.conn = dbus_bus_get(DBUS_BUS_SESSION, &err);
  if (!g_state.conn) {
    printf("tray: failed to connect to session bus: %s\n",
           err.message ? err.message : "unknown error");
    dbus_error_free(&err);
    return;
  }

  snprintf(g_state.well_known_name, sizeof(g_state.well_known_name),
           "org.kde.StatusNotifierItem-%d-1", (int)getpid());

  int flags =
      DBUS_NAME_FLAG_ALLOW_REPLACEMENT | DBUS_NAME_FLAG_REPLACE_EXISTING;
  dbus_bus_request_name(g_state.conn, g_state.well_known_name, flags, &err);
  if (dbus_error_is_set(&err)) {
    printf("tray: request_name failed: %s\n", err.message);
    dbus_error_free(&err);
  }

  DBusObjectPathVTable sni_vtable = {
      NULL, sni_message_handler, NULL, NULL, NULL, NULL};
  DBusObjectPathVTable menu_vtable = {
      NULL, menu_message_handler, NULL, NULL, NULL, NULL};
  DBusObjectPathVTable root_vtable = {
      NULL, root_message_handler, NULL, NULL, NULL, NULL};
  dbus_connection_try_register_object_path(g_state.conn, SNI_PATH, &sni_vtable,
                                           NULL, &err);
  dbus_connection_try_register_object_path(g_state.conn, MENU_PATH,
                                           &menu_vtable, NULL, &err);
  dbus_connection_try_register_object_path(g_state.conn, "/", &root_vtable,
                                           NULL, &err);

  dbus_bus_add_match(g_state.conn,
                     "type='signal',sender='org.freedesktop.DBus',interface='"
                     "org.freedesktop.DBus',member='NameOwnerChanged'",
                     &err);
  dbus_connection_add_filter(g_state.conn, name_owner_changed_filter, NULL,
                             NULL);

  wp_tray_files_spawn_watchers();

  pthread_t announce_thread;
  if (pthread_create(&announce_thread, NULL, tray_announce_thread_main, NULL) ==
      0) {
    pthread_detach(announce_thread);
  }
}
