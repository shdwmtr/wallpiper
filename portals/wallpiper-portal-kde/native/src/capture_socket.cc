#include "capture_socket.h"
#include "protocol.h"

#include <QDebug>

#include <sys/socket.h>
#include <unistd.h>

namespace WallpiperKde {

CaptureSocket::CaptureSocket(QObject *parent) : QObject(parent) {}

CaptureSocket::~CaptureSocket() { stop(); }

void CaptureSocket::start() {
  if (m_running.exchange(true)) {
    return;
  }
  m_thread = std::thread(&CaptureSocket::run, this);
}

void CaptureSocket::stop() {
  if (!m_running.exchange(false)) {
    return;
  }
  int fd = m_sockFd.exchange(-1);
  if (fd >= 0) {
    ::shutdown(fd, SHUT_RDWR);
    ::close(fd);
  }
  if (m_thread.joinable()) {
    m_thread.join();
  }
}

void CaptureSocket::run() {
  int sockFd =
      WallpiperProtocol::bindUnixDgram(WallpiperProtocol::kCaptureSocketPath);
  if (sockFd < 0) {
    qWarning() << "[capture] failed to bind"
               << WallpiperProtocol::kCaptureSocketPath;
    m_running = false;
    return;
  }
  m_sockFd = sockFd;
  qInfo() << "[capture] listening on" << WallpiperProtocol::kCaptureSocketPath
          << "(dgram)";

  while (m_running.load()) {
    auto received = WallpiperProtocol::recvMsg(sockFd);
    if (!received) {
      continue;
    }

    auto event = WallpiperProtocol::parseEvent(received->header, received->fds);
    if (!event) {
      qWarning() << "[capture] unrecognized or malformed message:"
                 << QString::fromStdString(received->header);
      for (int fd : received->fds) {
        ::close(fd);
      }
      continue;
    }

    if (const auto *buf = std::get_if<WallpiperProtocol::BufEvent>(&*event)) {
      emit bufReceived(buf->slot, buf->width, buf->height, buf->stride,
                       buf->modifier, buf->fd, buf->syncFd);
    } else if (const auto *frame =
                   std::get_if<WallpiperProtocol::FrameEvent>(&*event)) {
      emit frameReceived(frame->slot, frame->syncFd);
    } else if (const auto *shm =
                   std::get_if<WallpiperProtocol::ShmEvent>(&*event)) {
      emit shmReceived(shm->width, shm->height, shm->stride, shm->fd);
    }
  }
}

} // namespace WallpiperKde
