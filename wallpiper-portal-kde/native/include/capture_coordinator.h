#pragma once

#include "protocol.h"

#include <QObject>

#include <atomic>
#include <optional>
#include <vector>

QT_FORWARD_DECLARE_CLASS(QTimer)

namespace WallpiperKde
{

class CaptureSocket;
class CtlListener;
class WallpaperCaptureItem;

class CaptureCoordinator : public QObject
{
    Q_OBJECT
  public:
    static CaptureCoordinator* instance();

    void registerItem(WallpaperCaptureItem* item);
    void unregisterItem(WallpaperCaptureItem* item);

    void reevaluateActiveItem();

  private:
    explicit CaptureCoordinator(QObject* parent = nullptr);
    ~CaptureCoordinator() override;

    void ensureSocketsBound();
    void teardownSockets();

    std::optional<WallpiperProtocol::MonitorGeometry> geometryFromActiveItem() const;
    void handleDetach();
    void handleSetDebug(bool enabled);

    std::vector<WallpaperCaptureItem*> m_items;
    WallpaperCaptureItem* m_active = nullptr;

    CaptureSocket* m_captureSocket = nullptr;
    CtlListener* m_ctlListener = nullptr;

    QTimer* m_cursorTimer = nullptr;
    std::atomic<int32_t> m_cursorX{ 0 };
    std::atomic<int32_t> m_cursorY{ 0 };
};

} // namespace WallpiperKde
