// Host diagnostics that do not need the SDK.
#include <cstdio>
#include <cstdlib>
#include <string>
#include <vector>

#include "crlab/camera.hpp"

namespace crlab {

std::vector<std::string> competing_processes() {
  // Processes that may hold a PTP/USB or network session with the camera.
  // ptpcamerad/PTPCamera: macOS Image Capture daemon (reported by third
  // parties to claim PTP cameras). The rest are camera apps.
  static const char* names[] = {"ptpcamerad", "PTPCamera", "Imaging Edge Desktop",
                                "Capture One", "CaptureOne", "Image Capture", "Photos", "Lightroom", "Adobe Lightroom Classic"};
  std::vector<std::string> found;
  for (const char* n : names) {
    std::string cmd = std::string("/usr/bin/pgrep -xq '") + n + "' 2>/dev/null";
    if (std::system(cmd.c_str()) == 0) found.emplace_back(n);
  }
  return found;
}

}  // namespace crlab
