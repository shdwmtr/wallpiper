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

#include <X11/Xlib.h>
#include <X11/extensions/Xrandr.h>

namespace WallpiperKde {

namespace {
constexpr int kCursorSampleIntervalMs = 8;

/* must match layer/siphon/config.h */
constexpr uint32_t kCaptureSlotCount = 3;

QString x11OutputForSize(quint32 width, quint32 height) {
  static Display *dpy = XOpenDisplay(nullptr);
  if (!dpy) {
    return QString();
  }

  Window root = DefaultRootWindow(dpy);
  XRRScreenResources *res = XRRGetScreenResourcesCurrent(dpy, root);
  if (!res) {
    return QString();
  }

  QString found;
  for (int i = 0; i < res->noutput && found.isEmpty(); i++) {
    XRROutputInfo *outputInfo = XRRGetOutputInfo(dpy, res, res->outputs[i]);
    if (!outputInfo) {
      continue;
    }
    if (outputInfo->connection == RR_Connected && outputInfo->crtc != None) {
      XRRCrtcInfo *crtcInfo = XRRGetCrtcInfo(dpy, res, outputInfo->crtc);
      if (crtcInfo) {
        if (static_cast<quint32>(crtcInfo->width) == width &&
            static_cast<quint32>(crtcInfo->height) == height) {
          found = QString::fromUtf8(outputInfo->name);
        }
        XRRFreeCrtcInfo(crtcInfo);
      }
    }
    XRRFreeOutputInfo(outputInfo);
  }

  XRRFreeScreenResources(res);
  return found;
}
} // namespace

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
            uint32_t channel = slot / kCaptureSlotCount;
            WallpaperCaptureItem *item = channelItem(channel);
            if (!item) {
              item = claimItemForSize(channel, width, height);
            }
            if (item) {
              item->stageBuf(slot, width, height, stride, modifier, fd, syncFd);
            } else {
              ::close(fd);
              if (syncFd >= 0) {
                ::close(syncFd);
              }
            }
          });
  connect(m_captureSocket, &CaptureSocket::frameReceived, this,
          [this](quint32 slot, int syncFd) {
            uint32_t channel = slot / kCaptureSlotCount;
            if (WallpaperCaptureItem *item = channelItem(channel)) {
              item->stageFrame(slot, syncFd);
            } else if (syncFd >= 0) {
              ::close(syncFd);
            }
          });
  connect(m_captureSocket, &CaptureSocket::shmReceived, this,
          [this](quint32 width, quint32 height, quint32 stride, int fd) {
            if (m_primaryItem) {
              m_primaryItem->stageShm(width, height, stride, fd);
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
  for (auto channelIt = m_channelItems.begin();
       channelIt != m_channelItems.end();) {
    if (channelIt->second == item) {
      channelIt = m_channelItems.erase(channelIt);
    } else {
      ++channelIt;
    }
  }
  if (m_primaryItem == item) {
    m_primaryItem = nullptr;
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
  if (candidate == m_primaryItem) {
    return;
  }
  m_primaryItem = candidate;
  if (m_primaryItem) {
    qInfo() << "[coordinator] primary wallpaper item is now on screen"
            << (m_primaryItem->window() && m_primaryItem->window()->screen()
                    ? m_primaryItem->window()->screen()->name()
                    : QStringLiteral("<none>"));
  }
}

WallpaperCaptureItem *CaptureCoordinator::channelItem(uint32_t channel) const {
  auto it = m_channelItems.find(channel);
  return it == m_channelItems.end() ? nullptr : it->second;
}

WallpaperCaptureItem *CaptureCoordinator::claimItemForSize(uint32_t channel,
                                                           quint32 width,
                                                           quint32 height) {
  QString outputName = x11OutputForSize(width, height);
  if (outputName.isEmpty()) {
    qWarning() << "[coordinator] XRandR has no output currently sized" << width
               << "x" << height << "-- cannot bind channel" << channel;
    return nullptr;
  }

  for (auto *item : m_items) {
    bool alreadyClaimed = false;
    for (const auto &[boundChannel, boundItem] : m_channelItems) {
      if (boundItem == item) {
        alreadyClaimed = true;
        break;
      }
    }
    if (alreadyClaimed) {
      continue;
    }
    QScreen *screen = item->window() ? item->window()->screen() : nullptr;
    if (screen && screen->name() == outputName) {
      m_channelItems[channel] = item;
      qInfo() << "[coordinator] channel" << channel << "bound to screen"
              << outputName << "for stream" << width << "x" << height;
      return item;
    }
  }
  qWarning() << "[coordinator] XRandR output" << outputName << "(matched"
             << width << "x" << height
             << ") not found among registered wallpaper items, or already "
                "claimed -- cannot bind channel"
             << channel;
  return nullptr;
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
  if (!m_primaryItem) {
    return std::nullopt;
  }
  return m_primaryItem->currentGeometry();
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
