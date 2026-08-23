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

private:
  explicit CaptureCoordinator(QObject *parent = nullptr);
  ~CaptureCoordinator() override;

  void ensureSocketsBound();
  void teardownSockets();

  WallpaperCaptureItem *channelItem(uint32_t channel) const;
  WallpaperCaptureItem *claimItemForSize(uint32_t channel, quint32 width,
                                         quint32 height);

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
