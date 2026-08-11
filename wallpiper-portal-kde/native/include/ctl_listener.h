#pragma once

#include "protocol.h"

#include <QObject>
#include <QString>

#include <atomic>
#include <functional>
#include <optional>
#include <thread>
#include <utility>

namespace WallpiperKde
{

class CtlListener : public QObject
{
    Q_OBJECT
  public:
    using CursorPosProvider = std::function<std::optional<std::pair<int32_t, int32_t>>()>;
    using GeometryProvider = std::function<std::optional<WallpiperProtocol::MonitorGeometry>()>;
    using DetachHandler = std::function<void()>;
    using DebugHandler = std::function<void(bool)>;

    explicit CtlListener(QString portalName, QObject* parent = nullptr);
    ~CtlListener() override;

    void start();
    void stop();

    void setCursorPosProvider(CursorPosProvider provider);
    void setGeometryProvider(GeometryProvider provider);
    void setDetachHandler(DetachHandler handler);
    void setDebugHandler(DebugHandler handler);

  private:
    void run();
    void handleConnection(int clientFd);

    QString m_portalName;
    std::thread m_thread;
    std::atomic<int> m_listenFd{ -1 };
    std::atomic<bool> m_running{ false };

    CursorPosProvider m_cursorPosProvider;
    GeometryProvider m_geometryProvider;
    DetachHandler m_detachHandler;
    DebugHandler m_debugHandler;
};

} // namespace WallpiperKde
