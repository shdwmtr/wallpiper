#include "capture_coordinator.h"

#include "capture_item.h"
#include "capture_socket.h"
#include "ctl_listener.h"

#include <QCursor>
#include <QDebug>
#include <QGuiApplication>
#include <QScreen>
#include <QTimer>

#include <algorithm>

#include <unistd.h>

namespace WallpiperKde {

namespace {
constexpr int kCursorSampleIntervalMs = 8;
}

CaptureCoordinator *CaptureCoordinator::instance() {
  static CaptureCoordinator *coordinator = new CaptureCoordinator();
  return coordinator;
}

CaptureCoordinator::CaptureCoordinator(QObject *parent)
    : QObject(parent), m_captureSocket(new CaptureSocket(this)),
      m_ctlListener(new CtlListener(QStringLiteral("kde"), this)),
      m_cursorTimer(new QTimer(this)) {
  connect(m_captureSocket, &CaptureSocket::bufReceived, this,
          [this](quint32 slot, quint32 width, quint32 height, quint32 stride,
                 quint64 modifier, int fd, int syncFd) {
            if (m_active) {
              m_active->stageBuf(slot, width, height, stride, modifier, fd,
                                 syncFd);
            } else {
              ::close(fd);
              if (syncFd >= 0) {
                ::close(syncFd);
              }
            }
          });
  connect(m_captureSocket, &CaptureSocket::frameReceived, this,
          [this](quint32 slot, int syncFd) {
            if (m_active) {
              m_active->stageFrame(slot, syncFd);
            } else if (syncFd >= 0) {
              ::close(syncFd);
            }
          });
  connect(m_captureSocket, &CaptureSocket::shmReceived, this,
          [this](quint32 width, quint32 height, quint32 stride, int fd) {
            if (m_active) {
              m_active->stageShm(width, height, stride, fd);
            } else {
              ::close(fd);
            }
          });

  m_ctlListener->setCursorPosProvider(
      [this]() -> std::optional<std::pair<int32_t, int32_t>> {
        return std::make_pair(m_cursorX.load(), m_cursorY.load());
      });
  m_ctlListener->setGeometryProvider(
      [this]() -> std::optional<WallpiperProtocol::MonitorGeometry> {
        std::optional<WallpiperProtocol::MonitorGeometry> result;
        QMetaObject::invokeMethod(
            this, [this, &result]() { result = geometryFromActiveItem(); },
            Qt::BlockingQueuedConnection);
        return result;
      });
  m_ctlListener->setDetachHandler([this]() {
    QMetaObject::invokeMethod(
        this, [this]() { handleDetach(); }, Qt::BlockingQueuedConnection);
  });
  m_ctlListener->setDebugHandler([this](bool enabled) {
    QMetaObject::invokeMethod(
        this, [this, enabled]() { handleSetDebug(enabled); },
        Qt::BlockingQueuedConnection);
  });

  m_cursorTimer->setInterval(kCursorSampleIntervalMs);
  connect(m_cursorTimer, &QTimer::timeout, this, [this]() {
    QPoint pos = QCursor::pos();
    m_cursorX = pos.x();
    m_cursorY = pos.y();
  });
}

CaptureCoordinator::~CaptureCoordinator() { teardownSockets(); }

void CaptureCoordinator::registerItem(WallpaperCaptureItem *item) {
  if (!item ||
      std::find(m_items.begin(), m_items.end(), item) != m_items.end()) {
    return;
  }
  m_items.push_back(item);
  ensureSocketsBound();
  reevaluateActiveItem();
}

void CaptureCoordinator::unregisterItem(WallpaperCaptureItem *item) {
  auto it = std::find(m_items.begin(), m_items.end(), item);
  if (it == m_items.end()) {
    return;
  }
  m_items.erase(it);
  if (m_active == item) {
    m_active = nullptr;
  }
  reevaluateActiveItem();
  if (m_items.empty()) {
    teardownSockets();
  }
}

void CaptureCoordinator::reevaluateActiveItem() {
  WallpaperCaptureItem *candidate = nullptr;
  QScreen *primary = QGuiApplication::primaryScreen();
  for (auto *item : m_items) {
    if (item->window() && item->window()->screen() == primary) {
      candidate = item;
      break;
    }
  }
  if (!candidate && !m_items.empty()) {
    candidate = m_items.front();
  }
  if (candidate == m_active) {
    return;
  }
  if (m_active) {
    m_active->clearDisplay();
  }
  m_active = candidate;
  if (m_active) {
    qInfo() << "[coordinator] active wallpaper item is now on screen"
            << (m_active->window() && m_active->window()->screen()
                    ? m_active->window()->screen()->name()
                    : QStringLiteral("<none>"));
  }
}

void CaptureCoordinator::ensureSocketsBound() {
  m_captureSocket->start();
  m_ctlListener->start();
  if (!m_cursorTimer->isActive()) {
    m_cursorTimer->start();
  }
}

void CaptureCoordinator::teardownSockets() {
  m_captureSocket->stop();
  m_ctlListener->stop();
  m_cursorTimer->stop();
}

std::optional<WallpiperProtocol::MonitorGeometry>
CaptureCoordinator::geometryFromActiveItem() const {
  if (!m_active) {
    return std::nullopt;
  }
  return m_active->currentGeometry();
}

void CaptureCoordinator::handleDetach() {
  for (auto *item : m_items) {
    item->requestDetach();
  }
  qInfo() << "[ctl] detached, released all buffers";
}

void CaptureCoordinator::handleSetDebug(bool enabled) {
  for (auto *item : m_items) {
    item->setDebugEnabled(enabled);
  }
  qInfo() << "[ctl] debug overlay ->" << enabled;
}

} // namespace WallpiperKde
