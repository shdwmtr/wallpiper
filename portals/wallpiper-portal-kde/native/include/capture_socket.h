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
