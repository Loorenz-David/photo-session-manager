// SdkCameraAdapter: plugs CameraSession into the TransportManager.
// Compiled for both the real SDK and the stub (it only uses the facade).
#include <cstdio>
#include <cstdlib>
#include <unistd.h>

#include "crlab/camera.hpp"
#include "crlab/usb_monitor.hpp"

namespace crlab {

SdkCameraAdapter::SdkCameraAdapter(EventSink& log, ConnectOptions base, int discover_timeout_s)
    : log_(log), base_(std::move(base)), discover_timeout_s_(discover_timeout_s), session_(log) {}

std::vector<CameraEndpoint> SdkCameraAdapter::discover(Transport t) {
  std::vector<CameraEndpoint> out;
  // A previous helper run supplied this endpoint. The SDK can create its camera object directly.
  if (t == Transport::Wifi && !base_.ip.empty()) {
    out.push_back({base_.mac.empty() ? base_.ip : base_.mac, "ILCE-7RM5", Transport::Wifi, "direct-ip:" + base_.ip});
    return out;
  }
  int64_t dt = 0;
  last_found_ = CameraSession::discover(log_, discover_timeout_s_, &dt);
  for (const auto& c : last_found_) {
    if (c.transport != t) continue;
    out.push_back({c.id, c.model, c.transport, std::to_string(c.index)});
  }
  // Fallbacks when enumeration does not list the camera on this transport:
  // direct object creation by USB serial (from IOKit) or configured IP.
  if (out.empty() && t == Transport::Usb) {
    std::string serial = !base_.usb_serial.empty() ? base_.usb_serial : usb_serial_hint_;
    if (!serial.empty()) out.push_back({serial, "ILCE-7RM5", Transport::Usb, "direct-usb:" + serial});
  }
  if (out.empty() && t == Transport::Wifi && !base_.ip.empty()) {
    out.push_back({base_.mac.empty() ? base_.ip : base_.mac, "ILCE-7RM5", Transport::Wifi, "direct-ip:" + base_.ip});
  }
  return out;
}

void SdkCameraAdapter::forget_cached_endpoint() { base_.ip.clear(); base_.mac.clear(); }

ConnectResult SdkCameraAdapter::connect(const CameraEndpoint& ep) {
  ConnectOptions opt = base_;
  opt.transport = ep.transport;
  ConnectResult r;
  if (ep.address.rfind("direct-usb:", 0) == 0) {
    opt.usb_serial = ep.address.substr(11);
    opt.ip.clear();
    r = session_.connect(nullptr, opt);
  } else if (ep.address.rfind("direct-ip:", 0) == 0) {
    opt.usb_serial.clear();
    r = session_.connect(nullptr, opt);
  } else {
    int idx = std::atoi(ep.address.c_str());
    const DiscoveredCamera* cam = nullptr;
    for (const auto& c : last_found_)
      if (c.index == idx && c.transport == ep.transport) cam = &c;
    if (!cam) {
      r.message = "selected camera vanished from enumeration";
      return r;
    }
    r = session_.connect(cam, opt);
  }
  // Every failed USB connect (enumerated or direct by serial) says whether another process holds
  // the interface: the transport manager then falls back to Wi-Fi at once and waits for the
  // holder to let go (P56 rulings 1 and 2).
  if (!r.ok && ep.transport == Transport::Usb &&
      usb_held_by_other(report_usb_claim(log_, base_.usb_product_ids), static_cast<int>(getpid()))) {
    r.held = true;
    r.message += " (USB interface claimed by another process; see usb.claimed_by)";
  }
  if (r.ok) after_connect(r, ep.transport);
  if (r.ok && ep.transport == Transport::Wifi && ep.address.rfind("direct-ip:", 0) != 0) {
    // P63: remember the endpoint of this Wi-Fi session, so the reconnect after a lost session
    // (the cable unplugged) connects directly instead of waiting for a ~3.8 s discovery.
    int idx = std::atoi(ep.address.c_str());
    for (const auto& c : last_found_) {
      if (c.index == idx && c.transport == Transport::Wifi && !c.ip_address.empty()) {
        base_.ip = c.ip_address;
        base_.mac = !c.mac_address.empty() ? c.mac_address : c.id;
        log_.emit("endpoint.cached", Fields().str("transport", "wifi"));
      }
    }
  }
  return r;
}

void SdkCameraAdapter::after_connect(ConnectResult& r, Transport t) {
  // (1) Save-to-PC folder: a download on either link lands in the run's staging folder.
  if (!base_.save_dir.empty()) session_.configure_save(base_.save_dir, "");
  // (2) Store destination for this link, read back within 3 s. Host PC alone is never written.
  StoreDestinationResult sd = session_.set_store_destination(t == Transport::Usb ? "pc+camera" : "camera", 3000);
  // (3) Reported by TransportManager::try_connect in connection.ready.
  r.store_destination = sd.verified ? "verified" : "failed";
  if (sd.read) {
    char buf[16];
    std::snprintf(buf, sizeof buf, "0x%08llX", static_cast<unsigned long long>(sd.value));
    r.store_value = buf;
  }
}

void SdkCameraAdapter::before_voluntary_stop(Transport t) {
  if (t != Transport::Usb) return;
  // One attempt, at most 1 s: leave the camera on Memory Card for the next Wi-Fi session.
  session_.set_store_destination("camera", 1000);
}

void SdkCameraAdapter::disconnect() { session_.disconnect(); }

bool SdkCameraAdapter::read_state(Fields& out) {
  auto r = session_.read_properties(out, false);
  out.str("transport", to_string(session_.transport())).num("props_ms", r.duration_ms);
  return r.ok;
}

}  // namespace crlab
