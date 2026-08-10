#pragma once

#include <QJsonObject>
#include <QString>

#include <cstdint>
#include <optional>
#include <string>
#include <variant>
#include <vector>

namespace WallpiperProtocol {

constexpr const char *kCaptureSocketPath = "/tmp/wallpiper-capture.sock";

QString ctlSocketPath(const QString &portalName);

struct MonitorGeometry {
  int32_t x = 0;
  int32_t y = 0;
  uint32_t width = 0;
  uint32_t height = 0;
  uint32_t logicalWidth = 0;
  uint32_t logicalHeight = 0;

  bool operator==(const MonitorGeometry &other) const;
  bool operator!=(const MonitorGeometry &other) const;
};

QJsonObject toJson(const MonitorGeometry &geometry);
std::optional<MonitorGeometry> geometryFromJson(const QJsonObject &object);

struct BufEvent {
  uint32_t slot = 0;
  uint32_t width = 0;
  uint32_t height = 0;
  uint32_t stride = 0;
  uint64_t modifier = 0;
  int fd = -1;
  int syncFd = -1;
};

struct FrameEvent {
  uint32_t slot = 0;
  int syncFd = -1;
};

struct ShmEvent {
  uint32_t width = 0;
  uint32_t height = 0;
  uint32_t stride = 0;
  int fd = -1;
};

using SocketEvent = std::variant<BufEvent, FrameEvent, ShmEvent>;

std::optional<SocketEvent> parseEvent(const std::string &header,
                                      const std::vector<int> &fds);

struct RecvResult {
  std::string header;
  std::vector<int> fds;
};

std::optional<RecvResult> recvMsg(int sockFd);

int bindUnixDgram(const std::string &path);

int bindUnixStreamListener(const std::string &path, int backlog = 8);

enum class CtlRequest {
  Geometry,
  Detach,
  DebugOn,
  DebugOff,
  CursorPos,
};

std::optional<CtlRequest> parseCtlRequest(const std::string &line);
std::string encodeCtlRequest(CtlRequest request);

struct CtlResponseOk {};
struct CtlResponseErr {
  std::string message;
};
struct CtlResponseGeometry {
  MonitorGeometry geometry;
};
struct CtlResponseCursorPos {
  int32_t x = 0;
  int32_t y = 0;
};

using CtlResponse = std::variant<CtlResponseOk, CtlResponseErr,
                                 CtlResponseGeometry, CtlResponseCursorPos>;

std::string encodeCtlResponse(const CtlResponse &response);

} // namespace WallpiperProtocol
