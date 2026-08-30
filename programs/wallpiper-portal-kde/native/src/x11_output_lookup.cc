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

#include "x11_output_lookup.h"

#include <X11/Xlib.h>
#include <X11/extensions/Xrandr.h>

namespace WallpiperKde {

namespace {

Display *x11Display() {
  static Display *dpy = XOpenDisplay(nullptr);
  return dpy;
}

} // namespace

std::optional<QString> x11OutputForPosition(int32_t x, int32_t y) {
  Display *dpy = x11Display();
  if (!dpy) {
    return std::nullopt;
  }

  Window root = DefaultRootWindow(dpy);
  XRRScreenResources *res = XRRGetScreenResourcesCurrent(dpy, root);
  if (!res) {
    return std::nullopt;
  }

  std::optional<QString> result;
  for (int i = 0; i < res->noutput && !result; i++) {
    XRROutputInfo *outputInfo = XRRGetOutputInfo(dpy, res, res->outputs[i]);
    if (!outputInfo) {
      continue;
    }
    if (outputInfo->connection == RR_Connected && outputInfo->crtc != None) {
      XRRCrtcInfo *crtcInfo = XRRGetCrtcInfo(dpy, res, outputInfo->crtc);
      if (crtcInfo) {
        if (crtcInfo->x == x && crtcInfo->y == y) {
          result = QString::fromUtf8(outputInfo->name);
        }
        XRRFreeCrtcInfo(crtcInfo);
      }
    }
    XRRFreeOutputInfo(outputInfo);
  }

  XRRFreeScreenResources(res);
  return result;
}

std::optional<QString> x11OutputForSize(uint32_t width, uint32_t height) {
  Display *dpy = x11Display();
  if (!dpy) {
    return std::nullopt;
  }

  Window root = DefaultRootWindow(dpy);
  XRRScreenResources *res = XRRGetScreenResourcesCurrent(dpy, root);
  if (!res) {
    return std::nullopt;
  }

  std::optional<QString> result;
  for (int i = 0; i < res->noutput && !result; i++) {
    XRROutputInfo *outputInfo = XRRGetOutputInfo(dpy, res, res->outputs[i]);
    if (!outputInfo) {
      continue;
    }
    if (outputInfo->connection == RR_Connected && outputInfo->crtc != None) {
      XRRCrtcInfo *crtcInfo = XRRGetCrtcInfo(dpy, res, outputInfo->crtc);
      if (crtcInfo) {
        bool straight = static_cast<uint32_t>(crtcInfo->width) == width &&
                        static_cast<uint32_t>(crtcInfo->height) == height;
        bool rotated = static_cast<uint32_t>(crtcInfo->width) == height &&
                       static_cast<uint32_t>(crtcInfo->height) == width &&
                       (crtcInfo->rotation & (RR_Rotate_90 | RR_Rotate_270));
        if (straight || rotated) {
          result = QString::fromUtf8(outputInfo->name);
        }
        XRRFreeCrtcInfo(crtcInfo);
      }
    }
    XRRFreeOutputInfo(outputInfo);
  }

  XRRFreeScreenResources(res);
  return result;
}

} // namespace WallpiperKde
