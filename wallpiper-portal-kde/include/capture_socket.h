#pragma once

#include <QObject>

#include <atomic>
#include <thread>

namespace WallpiperKde {

class CaptureSocket : public QObject {
  Q_OBJECT
public:
  explicit CaptureSocket(QObject *parent = nullptr);
  ~CaptureSocket() override;

  void start();
  void stop();

signals:
  void bufReceived(quint32 slot, quint32 width, quint32 height, quint32 stride,
                   quint64 modifier, int fd, int syncFd);
  void frameReceived(quint32 slot, int syncFd);
  void shmReceived(quint32 width, quint32 height, quint32 stride, int fd);

private:
  void run();

  std::thread m_thread;
  std::atomic<int> m_sockFd{-1};
  std::atomic<bool> m_running{false};
};

} // namespace WallpiperKde
