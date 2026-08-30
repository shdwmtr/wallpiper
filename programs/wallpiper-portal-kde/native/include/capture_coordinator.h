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

#pragma once

#include "protocol.h"

#include <QObject>

#include <atomic>
#include <cstdint>
#include <optional>
#include <unordered_map>
#include <vector>

QT_FORWARD_DECLARE_CLASS(QTimer)

namespace WallpiperKde {

class CaptureSocket;
class CtlListener;
class WallpaperCaptureItem;

class CaptureCoordinator : public QObject {
  Q_OBJECT
public:
  static CaptureCoordinator *instance();

  void registerItem(WallpaperCaptureItem *item);
  void unregisterItem(WallpaperCaptureItem *item);
  void reevaluateActiveItem();
  bool captureChannel(uint32_t channel, const QString &path, QString &err);

private:
  explicit CaptureCoordinator(QObject *parent = nullptr);
  ~CaptureCoordinator() override;

  void ensureSocketsBound();
  void teardownSockets();

  WallpaperCaptureItem *channelItem(uint32_t channel) const;
  WallpaperCaptureItem *claimItemForPosition(uint32_t channel, qint32 x,
                                             qint32 y);
  WallpaperCaptureItem *claimItemForSize(uint32_t channel, quint32 width,
                                         quint32 height);
  WallpaperCaptureItem *claimItemByOutputName(uint32_t channel,
                                              const QString &name);

  std::optional<WallpiperProtocol::MonitorGeometry>
  geometryFromActiveItem() const;
  void handleDetach();
  void handleSetDebug(bool enabled);

  std::vector<WallpaperCaptureItem *> m_items;
  std::unordered_map<uint32_t, WallpaperCaptureItem *> m_channelItems;
  WallpaperCaptureItem *m_primaryItem = nullptr;

  CaptureSocket *m_captureSocket = nullptr;
  CtlListener *m_ctlListener = nullptr;

  QTimer *m_cursorTimer = nullptr;
  std::atomic<int32_t> m_cursorX{0};
  std::atomic<int32_t> m_cursorY{0};
};

} // namespace WallpiperKde
