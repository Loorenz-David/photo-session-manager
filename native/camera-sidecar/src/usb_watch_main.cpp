// crlab-usb-watch: report Sony USB device arrival/removal as JSON lines.
// Builds without the Sony SDK. Use it to verify that macOS sees the A7R V
// when the cable is plugged in, before any SDK session is attempted.
#include <csignal>
#include <cstdio>
#include <cstring>
#include <string>
#include <thread>

#include "crlab/json_log.hpp"
#include "crlab/usb_monitor.hpp"

static volatile std::sig_atomic_t g_stop = 0;
static void on_sig(int) { g_stop = 1; }

int main(int argc, char** argv) {
  std::string experiment = "usb-watch";
  std::string log_path;
  for (int i = 1; i < argc; ++i) {
    if (!std::strcmp(argv[i], "--experiment") && i + 1 < argc) experiment = argv[++i];
    else if (!std::strcmp(argv[i], "--log") && i + 1 < argc) log_path = argv[++i];
    else {
      std::fprintf(stderr, "usage: crlab-usb-watch [--experiment ID] [--log FILE]\n");
      return 2;
    }
  }
  crlab::JsonLog log(experiment, "usb");
  if (!log_path.empty() && !log.open_file(log_path)) {
    std::fprintf(stderr, "cannot open %s\n", log_path.c_str());
    return 2;
  }
  std::signal(SIGINT, on_sig);
  std::signal(SIGTERM, on_sig);

  crlab::UsbMonitor mon;
  bool ok = mon.start([&](bool arrived, const crlab::UsbDeviceInfo& d) {
    crlab::Fields f;
    f.hex("vendor_id", d.vendor_id).hex("product_id", d.product_id).hex("location_id", d.location_id)
        .str("product", d.product).boolean("serial_present", !d.serial.empty());
    log.emit(arrived ? "usb.arrived" : "usb.removed", f);
  });
  if (!ok) {
    log.emit("usb.monitor.error", crlab::Fields().str("message", "IOKit notification setup failed"));
    return 1;
  }
  log.emit("usb.monitor.started", crlab::Fields().hex("vendor_id", crlab::UsbMonitor::kSonyVendorId)
                                      .num("present", mon.present_count()));
  crlab::report_usb_claim(log);
  std::fprintf(stderr, "[usb-watch] watching for Sony USB devices; Ctrl-C to stop\n");
  while (!g_stop) std::this_thread::sleep_for(std::chrono::milliseconds(200));
  mon.stop();
  log.emit("usb.monitor.stopped");
  return 0;
}
