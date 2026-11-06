// macOS USB hot-plug monitor for Sony devices (IOKit, no SDK dependency).
//
// Uses IOServiceAddMatchingNotification on IOUSBHostDevice filtered by the
// Sony USB vendor id (0x054C). This tells us a Sony device is physically
// present; it does NOT tell us the camera is in "Remote Shooting" USB mode.
// Whether the SDK can open a session is decided by the SDK connect attempt.
#pragma once

#include <cstdint>
#include <functional>
#include <memory>
#include <string>
#include <vector>

namespace crlab {

struct UsbDeviceInfo {
  uint16_t vendor_id = 0;
  uint16_t product_id = 0;
  uint32_t location_id = 0;
  std::string product;
  std::string serial;
};

class UsbMonitor {
 public:
  static constexpr uint16_t kSonyVendorId = 0x054C;
  using Callback = std::function<void(bool arrived, const UsbDeviceInfo& info)>;

  // product_ids: the camera's USB product ids (R-I23: the match names the camera, not every Sony
  // device). A device of another product is ignored before it is counted. Empty = any product.
  explicit UsbMonitor(uint16_t vendor_id = kSonyVendorId, std::vector<uint16_t> product_ids = {});
  ~UsbMonitor();
  UsbMonitor(const UsbMonitor&) = delete;
  UsbMonitor& operator=(const UsbMonitor&) = delete;

  // Starts a background CFRunLoop thread. Devices already attached are
  // reported as arrivals immediately. Returns false if IOKit setup failed.
  bool start(Callback cb);
  void stop();
  int present_count() const;

  struct Impl;

 private:
  std::unique_ptr<Impl> impl_;
};

// Processes that hold a USB interface of a `vendor_id` device exclusively, as IOKit reports them
// ("pid 95408, ptpcamerad"). Observed 2026-11-05: with Capture One running (it links
// ImageCaptureCore), macOS's ptpcamerad claimed the A7R V on plug-in and every SDK USB connect
// failed within ~10 ms with CrError_Connect_TimeOut (0x8208). Empty when nothing claims it.
// product_ids filters the same way as UsbMonitor (empty = any product of the vendor).
std::vector<std::string> usb_exclusive_owners(uint16_t vendor_id = UsbMonitor::kSonyVendorId,
                                              const std::vector<uint16_t>& product_ids = {});

// Emits `usb.claimed_by` on `log` when some process holds the camera's USB interface.
// Returns the owners (empty if none).
class EventSink;
std::vector<std::string> report_usb_claim(EventSink& log, const std::vector<uint16_t>& product_ids = {});

}  // namespace crlab
