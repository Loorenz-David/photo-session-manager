#include "crlab/usb_monitor.hpp"

#include "crlab/json_log.hpp"

#include <CoreFoundation/CoreFoundation.h>
#include <IOKit/IOKitLib.h>
#include <IOKit/usb/IOUSBLib.h>

#include <atomic>
#include <condition_variable>
#include <mutex>
#include <thread>

namespace crlab {

namespace {

std::string cf_string_prop(io_service_t svc, const char* key) {
  CFStringRef k = CFStringCreateWithCString(kCFAllocatorDefault, key, kCFStringEncodingUTF8);
  CFTypeRef v = IORegistryEntryCreateCFProperty(svc, k, kCFAllocatorDefault, 0);
  CFRelease(k);
  std::string out;
  if (v && CFGetTypeID(v) == CFStringGetTypeID()) {
    char buf[256];
    if (CFStringGetCString(static_cast<CFStringRef>(v), buf, sizeof buf, kCFStringEncodingUTF8)) out = buf;
  }
  if (v) CFRelease(v);
  return out;
}

uint32_t cf_int_prop(io_service_t svc, const char* key) {
  CFStringRef k = CFStringCreateWithCString(kCFAllocatorDefault, key, kCFStringEncodingUTF8);
  CFTypeRef v = IORegistryEntryCreateCFProperty(svc, k, kCFAllocatorDefault, 0);
  CFRelease(k);
  uint32_t out = 0;
  if (v && CFGetTypeID(v) == CFNumberGetTypeID()) CFNumberGetValue(static_cast<CFNumberRef>(v), kCFNumberSInt32Type, &out);
  if (v) CFRelease(v);
  return out;
}

UsbDeviceInfo describe(io_service_t svc) {
  UsbDeviceInfo i;
  i.vendor_id = static_cast<uint16_t>(cf_int_prop(svc, "idVendor"));
  i.product_id = static_cast<uint16_t>(cf_int_prop(svc, "idProduct"));
  i.location_id = cf_int_prop(svc, "locationID");
  i.product = cf_string_prop(svc, "USB Product Name");
  if (i.product.empty()) i.product = cf_string_prop(svc, "kUSBProductString");
  i.serial = cf_string_prop(svc, "USB Serial Number");
  if (i.serial.empty()) i.serial = cf_string_prop(svc, "kUSBSerialNumberString");
  return i;
}

// The product filter (R-I23). `product_id` 0 means the registry entry did not report one: kept,
// so a missing property never hides the camera.
bool product_accepted(const std::vector<uint16_t>& products, uint16_t product_id) {
  if (products.empty() || product_id == 0) return true;
  for (uint16_t p : products)
    if (p == product_id) return true;
  return false;
}

}  // namespace

struct UsbMonitor::Impl {
  uint16_t vendor;
  std::vector<uint16_t> products;  // empty = any product of `vendor`
  Callback cb;
  std::thread thread;
  CFRunLoopRef loop = nullptr;
  IONotificationPortRef port = nullptr;
  io_iterator_t added_iter = 0;
  io_iterator_t removed_iter = 0;
  std::atomic<int> present{0};
  std::mutex mu;
  std::condition_variable cv;
  bool ready = false;
  bool ok = false;

  static void on_added(void* refcon, io_iterator_t it) {
    auto* self = static_cast<Impl*>(refcon);
    while (io_service_t svc = IOIteratorNext(it)) {
      auto info = describe(svc);
      // Filter before counting, so `usb.initial` and arrivals agree (R-I23).
      if (!product_accepted(self->products, info.product_id)) {
        IOObjectRelease(svc);
        continue;
      }
      ++self->present;
      if (self->cb) self->cb(true, info);
      IOObjectRelease(svc);
    }
  }
  static void on_removed(void* refcon, io_iterator_t it) {
    auto* self = static_cast<Impl*>(refcon);
    while (io_service_t svc = IOIteratorNext(it)) {
      auto info = describe(svc);
      if (!product_accepted(self->products, info.product_id)) {
        IOObjectRelease(svc);
        continue;
      }
      if (self->present > 0) --self->present;
      if (self->cb) self->cb(false, info);
      IOObjectRelease(svc);
    }
  }

  CFMutableDictionaryRef matching() const {
    // IOUSBHostDevice is the modern (macOS 10.11+) USB device class.
    // idVendor must go inside IOPropertyMatch: as a top-level key it is handled by the USB
    // family's matching rules, which reject idVendor without idProduct, so nothing ever matched
    // (observed 2026-11-05: an attached ILCE-7RM5, idVendor 1356, reported present=0).
    CFMutableDictionaryRef d = IOServiceMatching("IOUSBHostDevice");
    CFMutableDictionaryRef props = CFDictionaryCreateMutable(kCFAllocatorDefault, 0, &kCFTypeDictionaryKeyCallBacks,
                                                             &kCFTypeDictionaryValueCallBacks);
    int32_t vid = vendor;
    CFNumberRef n = CFNumberCreate(kCFAllocatorDefault, kCFNumberSInt32Type, &vid);
    CFDictionarySetValue(props, CFSTR("idVendor"), n);
    CFDictionarySetValue(d, CFSTR(kIOPropertyMatchKey), props);
    CFRelease(n);
    CFRelease(props);
    return d;
  }

  void run() {
    loop = CFRunLoopGetCurrent();
    port = IONotificationPortCreate(kIOMainPortDefault);
    bool setup = port != nullptr;
    if (setup) {
      CFRunLoopAddSource(loop, IONotificationPortGetRunLoopSource(port), kCFRunLoopDefaultMode);
      // IOServiceAddMatchingNotification consumes one reference per call.
      kern_return_t a = IOServiceAddMatchingNotification(port, kIOFirstMatchNotification, matching(), on_added, this, &added_iter);
      kern_return_t r = IOServiceAddMatchingNotification(port, kIOTerminatedNotification, matching(), on_removed, this, &removed_iter);
      setup = (a == KERN_SUCCESS && r == KERN_SUCCESS);
      if (setup) {
        on_added(this, added_iter);     // arm + report devices already attached
        on_removed(this, removed_iter);  // arm
      }
    }
    {
      std::lock_guard<std::mutex> lk(mu);
      ok = setup;
      ready = true;
    }
    cv.notify_all();
    if (setup) CFRunLoopRun();
    if (added_iter) IOObjectRelease(added_iter);
    if (removed_iter) IOObjectRelease(removed_iter);
    if (port) IONotificationPortDestroy(port);
    added_iter = removed_iter = 0;
    port = nullptr;
  }
};

UsbMonitor::UsbMonitor(uint16_t vendor_id, std::vector<uint16_t> product_ids) : impl_(std::make_unique<Impl>()) {
  impl_->vendor = vendor_id;
  impl_->products = std::move(product_ids);
}

UsbMonitor::~UsbMonitor() { stop(); }

bool UsbMonitor::start(Callback cb) {
  impl_->cb = std::move(cb);
  impl_->thread = std::thread([this] { impl_->run(); });
  std::unique_lock<std::mutex> lk(impl_->mu);
  impl_->cv.wait(lk, [this] { return impl_->ready; });
  return impl_->ok;
}

void UsbMonitor::stop() {
  if (impl_ && impl_->thread.joinable()) {
    if (impl_->loop) CFRunLoopStop(impl_->loop);
    impl_->thread.join();
  }
}

int UsbMonitor::present_count() const { return impl_->present.load(); }

std::vector<std::string> usb_exclusive_owners(uint16_t vendor_id, const std::vector<uint16_t>& product_ids) {
  std::vector<std::string> out;
  CFMutableDictionaryRef d = IOServiceMatching("IOUSBHostInterface");
  CFMutableDictionaryRef props = CFDictionaryCreateMutable(kCFAllocatorDefault, 0, &kCFTypeDictionaryKeyCallBacks,
                                                           &kCFTypeDictionaryValueCallBacks);
  int32_t vid = vendor_id;
  CFNumberRef n = CFNumberCreate(kCFAllocatorDefault, kCFNumberSInt32Type, &vid);
  CFDictionarySetValue(props, CFSTR("idVendor"), n);
  CFDictionarySetValue(d, CFSTR(kIOPropertyMatchKey), props);
  CFRelease(n);
  CFRelease(props);
  io_iterator_t it = 0;
  if (IOServiceGetMatchingServices(kIOMainPortDefault, d, &it) != KERN_SUCCESS) return out;  // consumes d
  while (io_service_t svc = IOIteratorNext(it)) {
    if (!product_accepted(product_ids, static_cast<uint16_t>(cf_int_prop(svc, "idProduct")))) {
      IOObjectRelease(svc);
      continue;
    }
    std::string owner = cf_string_prop(svc, "UsbExclusiveOwner");
    if (!owner.empty()) out.push_back(owner);
    IOObjectRelease(svc);
  }
  IOObjectRelease(it);
  return out;
}

std::vector<std::string> report_usb_claim(EventSink& log, const std::vector<uint16_t>& product_ids) {
  auto owners = usb_exclusive_owners(UsbMonitor::kSonyVendorId, product_ids);
  if (!owners.empty()) {
    std::string joined;
    for (const auto& o : owners) joined += (joined.empty() ? "" : "; ") + o;
    log.emit("usb.claimed_by", Fields().str("owners", joined).str(
        "hint", "another process holds the camera's USB interface; the SDK cannot open it. ptpcamerad claims "
                "cameras for apps using Image Capture (e.g. Capture One, Photos). Quit that app, then replug."));
  }
  return owners;
}

}  // namespace crlab
