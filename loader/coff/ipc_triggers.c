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

#include "ipc_triggers.h"
#include "cursor.h"
#include "menu.h"
#include "tray.h"
#include "util.h"
#include "libc.h"

#define NOTIFY_BUF_SIZE 4096

typedef struct {
  WCHAR command_path[MAX_PATH];
  WCHAR command_fname[MAX_PATH];
  BOOL have_command_path;

  WCHAR click_path[MAX_PATH];
  WCHAR click_fname[MAX_PATH];
  BOOL have_click_path;
} trigger_config_t;

static BOOL load_trigger_config(trigger_config_t *cfg) {
  cfg->command_fname[0] = L'\0';
  cfg->click_fname[0] = L'\0';

  cfg->have_command_path =
      get_env_path(L"WALLPIPER_MENU_COMMAND_FILE", cfg->command_path, MAX_PATH);
  if (cfg->have_command_path) {
    basename_w(cfg->command_path, cfg->command_fname);
  }

  cfg->have_click_path =
      get_env_path(L"WALLPIPER_TRAY_CLICK_FILE", cfg->click_path, MAX_PATH);
  if (cfg->have_click_path) {
    basename_w(cfg->click_path, cfg->click_fname);
  }

  return cfg->have_command_path || cfg->have_click_path;
}

static BOOL resolve_watch_directory(const trigger_config_t *cfg,
                                    WCHAR *out_dir) {
  lstrcpyW(out_dir,
           cfg->have_command_path ? cfg->command_path : cfg->click_path);

  WCHAR *last_sep = NULL;
  for (WCHAR *p = out_dir; *p; p++) {
    if (*p == L'\\' || *p == L'/') {
      last_sep = p;
    }
  }
  if (!last_sep) {
    return FALSE;
  }
  *last_sep = L'\0';
  return TRUE;
}

static HANDLE open_watch_handle(const WCHAR *dir) {
  return CreateFileW(dir, FILE_LIST_DIRECTORY,
                     FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
                     NULL, OPEN_EXISTING, FILE_FLAG_BACKUP_SEMANTICS, NULL);
}

static void scan_notify_buffer(const BYTE *notify_buf,
                               const trigger_config_t *cfg,
                               BOOL *matched_command, BOOL *matched_click) {
  DWORD offset = 0;
  for (;;) {
    const FILE_NOTIFY_INFORMATION *info =
        (const FILE_NOTIFY_INFORMATION *)(notify_buf + offset);

    int len = (int)(info->FileNameLength / sizeof(WCHAR));
    if (len >= MAX_PATH) {
      len = MAX_PATH - 1;
    }
    WCHAR namebuf[MAX_PATH];
    memcpy(namebuf, info->FileName, (size_t)len * sizeof(WCHAR));
    namebuf[len] = L'\0';

    BOOL is_write = info->Action == FILE_ACTION_ADDED ||
                    info->Action == FILE_ACTION_MODIFIED ||
                    info->Action == FILE_ACTION_RENAMED_NEW_NAME;
    if (is_write && cfg->have_command_path &&
        lstrcmpiW(namebuf, cfg->command_fname) == 0) {
      *matched_command = TRUE;
    }
    if (is_write && cfg->have_click_path &&
        lstrcmpiW(namebuf, cfg->click_fname) == 0) {
      *matched_click = TRUE;
    }

    if (info->NextEntryOffset == 0) {
      break;
    }
    offset += info->NextEntryOffset;
  }
}

static void handle_click_trigger(const WCHAR *click_path) {
  HANDLE hf = CreateFileW(click_path, GENERIC_READ, FILE_SHARE_READ, NULL,
                          OPEN_EXISTING, FILE_ATTRIBUTE_NORMAL, NULL);
  if (hf == INVALID_HANDLE_VALUE) {
    return;
  }

  UINT32 event_code = 0;
  DWORD read_bytes = 0;
  BOOL rok = ReadFile(hf, &event_code, 4, &read_bytes, NULL);
  CloseHandle(hf);
  DeleteFileW(click_path);

  if (!rok || read_bytes != 4 || !g_tray_hwnd) {
    return;
  }

  post_tray_click(event_code != 0);

  char logbuf[80];
  wsprintfA(logbuf, "handle_click_trigger: forwarded event_code=%lu to hwnd=%p",
            (unsigned long)event_code, (void *)(ULONG_PTR)g_tray_hwnd);
  debug_log(logbuf);
}

static void handle_command_trigger(const WCHAR *command_path) {
  HANDLE hf = CreateFileW(command_path, GENERIC_READ, FILE_SHARE_READ, NULL,
                          OPEN_EXISTING, FILE_ATTRIBUTE_NORMAL, NULL);
  if (hf == INVALID_HANDLE_VALUE) {
    return;
  }

  UINT32 command_id = 0;
  DWORD read_bytes = 0;
  BOOL rok = ReadFile(hf, &command_id, 4, &read_bytes, NULL);
  CloseHandle(hf);
  DeleteFileW(command_path);

  if (!rok || read_bytes != 4 || !g_menu_owner_hwnd) {
    return;
  }

  PostMessageW((HWND)(ULONG_PTR)g_menu_owner_hwnd, WM_COMMAND,
               MAKEWPARAM((WORD)command_id, 0), 0);

  char logbuf[80];
  wsprintfA(logbuf,
            "handle_command_trigger: posted WM_COMMAND id=%lu to hwnd=%p",
            (unsigned long)command_id, (void *)(ULONG_PTR)g_menu_owner_hwnd);
  debug_log(logbuf);
}

static void watch_loop(HANDLE hDir, const trigger_config_t *cfg,
                       BYTE *notify_buf) {
  for (;;) {
    DWORD bytes = 0;
    BOOL ok = ReadDirectoryChangesW(hDir, notify_buf, NOTIFY_BUF_SIZE, FALSE,
                                    FILE_NOTIFY_CHANGE_FILE_NAME |
                                        FILE_NOTIFY_CHANGE_LAST_WRITE,
                                    &bytes, NULL, NULL);
    if (!ok) {
      debug_log("watch_loop: ReadDirectoryChangesW failed, retrying");
      Sleep(1000);
      continue;
    }
    if (bytes == 0) {
      continue;
    }

    BOOL matched_command = FALSE;
    BOOL matched_click = FALSE;
    scan_notify_buffer(notify_buf, cfg, &matched_command, &matched_click);

    if (matched_click) {
      handle_click_trigger(cfg->click_path);
    }
    if (matched_command) {
      handle_command_trigger(cfg->command_path);
    }
  }
}

DWORD WINAPI file_trigger_watcher_thread(LPVOID param) {
  (void)param;

  trigger_config_t cfg;
  if (!load_trigger_config(&cfg)) {
    debug_log("file_trigger_watcher_thread: no trigger files configured, not "
              "starting");
    return 0;
  }

  WCHAR watch_dir[MAX_PATH];
  if (!resolve_watch_directory(&cfg, watch_dir)) {
    debug_log("file_trigger_watcher_thread: trigger file path has no directory "
              "component");
    return 0;
  }

  HANDLE hDir = open_watch_handle(watch_dir);
  if (hDir == INVALID_HANDLE_VALUE) {
    debug_log("file_trigger_watcher_thread: CreateFileW(directory) failed");
    return 0;
  }

  BYTE *notify_buf = (BYTE *)malloc(NOTIFY_BUF_SIZE);
  if (!notify_buf) {
    debug_log("file_trigger_watcher_thread: failed to allocate notify buffer");
    CloseHandle(hDir);
    return 0;
  }

  debug_log(
      "file_trigger_watcher_thread: started, blocked on ReadDirectoryChangesW");
  watch_loop(hDir, &cfg, notify_buf);

  return 0;
}
