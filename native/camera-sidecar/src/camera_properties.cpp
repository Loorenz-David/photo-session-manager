// Property reading (GetDeviceProperties) with best-effort value decoding.
//
// Decoding conventions (F-number x100, shutter speed num<<16|den, ISO in the
// low 24 bits) follow the RemoteCli sample's PropertyValueTable. Every value
// is ALSO reported raw so nothing depends on these conventions being right.
#include <cstdio>
#include <vector>

#include "sdk_internal.hpp"

namespace crlab {

namespace {

enum class Dec { Raw, FNumber, Shutter, Iso, Percent };

struct PropSpec {
  uint32_t code;
  const char* name;
  Dec dec;
  bool ftp;
};

#define P(SYM, NAME, DEC, FTP) {static_cast<uint32_t>(SDK::CrDeviceProperty_##SYM), NAME, DEC, FTP},
const PropSpec kProps[] = {
#if CRLAB_HAS_PROP_ExposureProgramMode
    P(ExposureProgramMode, "exposure_program", Dec::Raw, false)
#endif
#if CRLAB_HAS_PROP_FNumber
    P(FNumber, "f_number", Dec::FNumber, false)
#endif
#if CRLAB_HAS_PROP_ShutterSpeed
    P(ShutterSpeed, "shutter_speed", Dec::Shutter, false)
#endif
#if CRLAB_HAS_PROP_IsoSensitivity
    P(IsoSensitivity, "iso", Dec::Iso, false)
#endif
#if CRLAB_HAS_PROP_WhiteBalance
    P(WhiteBalance, "white_balance", Dec::Raw, false)
#endif
#if CRLAB_HAS_PROP_DriveMode
    P(DriveMode, "drive_mode", Dec::Raw, false)
#endif
#if CRLAB_HAS_PROP_FileType
    P(FileType, "file_type", Dec::Raw, false)
#endif
#if CRLAB_HAS_PROP_MediaSLOT1_FileType
    P(MediaSLOT1_FileType, "slot1_file_type", Dec::Raw, false)
#endif
#if CRLAB_HAS_PROP_StillImageQuality
    P(StillImageQuality, "still_image_quality", Dec::Raw, false)
#endif
#if CRLAB_HAS_PROP_RAW_FileCompressionType
    P(RAW_FileCompressionType, "raw_compression", Dec::Raw, false)
#endif
#if CRLAB_HAS_PROP_BatteryRemain
    P(BatteryRemain, "battery_remain_pct", Dec::Percent, false)
#endif
#if CRLAB_HAS_PROP_BatteryLevel
    P(BatteryLevel, "battery_level", Dec::Raw, false)
#endif
#if CRLAB_HAS_PROP_StillImageStoreDestination
    P(StillImageStoreDestination, "still_image_store_destination", Dec::Raw, false)
#endif
#if CRLAB_HAS_PROP_RAW_J_PC_Save_Image
    P(RAW_J_PC_Save_Image, "pc_save_image", Dec::Raw, false)
#endif
#if CRLAB_HAS_PROP_MediaSLOT1_Status
    P(MediaSLOT1_Status, "slot1_status", Dec::Raw, false)
#endif
#if CRLAB_HAS_PROP_MediaSLOT2_Status
    P(MediaSLOT2_Status, "slot2_status", Dec::Raw, false)
#endif
#if CRLAB_HAS_PROP_MediaSLOT1_RemainingNumber
    P(MediaSLOT1_RemainingNumber, "slot1_remaining_shots", Dec::Raw, false)
#endif
#if CRLAB_HAS_PROP_RecordingMedia
    P(RecordingMedia, "recording_media", Dec::Raw, false)
#endif
#if CRLAB_HAS_PROP_SdkControlMode
    P(SdkControlMode, "sdk_control_mode", Dec::Raw, false)
#endif
#if CRLAB_HAS_PROP_FTP_Function
    P(FTP_Function, "ftp_function", Dec::Raw, true)
#endif
#if CRLAB_HAS_PROP_FTP_AutoTransfer
    P(FTP_AutoTransfer, "ftp_auto_transfer", Dec::Raw, true)
#endif
#if CRLAB_HAS_PROP_FTP_TransferTarget
    P(FTP_TransferTarget, "ftp_transfer_target", Dec::Raw, true)
#endif
#if CRLAB_HAS_PROP_FTP_ConnectionStatus
    P(FTP_ConnectionStatus, "ftp_connection_status", Dec::Raw, true)
#endif
#if CRLAB_HAS_PROP_FTP_ConnectionErrorInfo
    P(FTP_ConnectionErrorInfo, "ftp_connection_error", Dec::Raw, true)
#endif
#if CRLAB_HAS_PROP_FTP_AutoTransferTarget
    P(FTP_AutoTransferTarget, "ftp_auto_transfer_target", Dec::Raw, true)
#endif
#if CRLAB_HAS_PROP_FTP_AutoTransferTarget_StillImage
    P(FTP_AutoTransferTarget_StillImage, "ftp_auto_transfer_target_still", Dec::Raw, true)
#endif
#if CRLAB_HAS_PROP_FTP_PowerSave
    P(FTP_PowerSave, "ftp_power_save", Dec::Raw, true)
#endif
    {0, nullptr, Dec::Raw, false},  // sentinel (keeps the array non-empty)
};
#undef P

std::string decode(Dec d, uint64_t v) {
  char b[64];
  switch (d) {
    case Dec::FNumber:
      if (v >= 0xFFFD) return "special";
      std::snprintf(b, sizeof b, "F%.1f", static_cast<double>(v) / 100.0);
      return b;
    case Dec::Shutter: {
      if (v == 0) return "BULB";
      uint32_t num = static_cast<uint32_t>(v >> 16) & 0xFFFF, den = static_cast<uint32_t>(v) & 0xFFFF;
      if (den == 0) return "special";
      if (num == 1 && den > 1) std::snprintf(b, sizeof b, "1/%u", den);
      else std::snprintf(b, sizeof b, "%.1f\"", static_cast<double>(num) / den);
      return b;
    }
    case Dec::Iso: {
      uint32_t val = static_cast<uint32_t>(v) & 0x00FFFFFF;
      if (val == 0xFFFFFF) return "AUTO";
      std::snprintf(b, sizeof b, "ISO %u", val);
      return b;
    }
    case Dec::Percent:
      std::snprintf(b, sizeof b, "%llu%%", static_cast<unsigned long long>(v));
      return b;
    default:
      return "";
  }
}

std::string prop_json(const SDK::CrDeviceProperty& p, Dec d) {
  auto& mp = const_cast<SDK::CrDeviceProperty&>(p);
  uint64_t v = mp.GetCurrentValue();
  Fields f;
  f.hex("code", mp.GetCode()).unum("value", v).hex("value_hex", v);
  std::string disp = decode(d, v);
  if (!disp.empty()) f.str("display", disp);
  f.boolean("writable", mp.IsSetEnableCurrentValue()).boolean("readable", mp.IsGetEnableCurrentValue());
  return f.encode_object();
}

}  // namespace

PropertyReadResult read_props_filtered(CameraSession::Impl& im, Fields& out, bool all, bool ftp_only) {
  PropertyReadResult r;
  SDK::CrDeviceProperty* props = nullptr;
  CrInt32 n = 0;
  int64_t t0 = monotonic_ms();
  auto err = SDK::GetDeviceProperties(im.handle, &props, &n);
  r.duration_ms = monotonic_ms() - t0;
  r.sdk_error = err;
  if (err != SDK::CrError_None || !props) {
    if (props) SDK::ReleaseDeviceProperties(im.handle, props);
    return r;
  }
  r.ok = true;
  std::string unknown = "{";
  bool first_unknown = true;
  for (CrInt32 i = 0; i < n; ++i) {
    uint32_t code = props[i].GetCode();
    const PropSpec* spec = nullptr;
    for (const auto& s : kProps)
      if (s.name && s.code == code) spec = &s;
    if (spec && (!ftp_only || spec->ftp)) {
      out.raw(spec->name, prop_json(props[i], spec->dec));
      ++r.count;
    } else if (all && !spec) {
      char key[16];
      std::snprintf(key, sizeof key, "0x%04X", code);
      unknown += std::string(first_unknown ? "" : ",") + "\"" + key + "\":" + prop_json(props[i], Dec::Raw);
      first_unknown = false;
      ++r.count;
    }
  }
  if (all) out.raw("other_codes", unknown + "}");
  SDK::ReleaseDeviceProperties(im.handle, props);
  return r;
}

PropertyReadResult CameraSession::read_properties(Fields& out, bool all) {
  if (!connected()) return {};
  auto r = read_props_filtered(*impl_, out, all, false);
  impl_->log.emit("props.read", Fields().boolean("success", r.ok).hex("sdk_error", r.sdk_error)
                                    .num("duration_ms", r.duration_ms).num("count", r.count)
                                    .str("transport", to_string(impl_->transport)));
  return r;
}

PropertyReadResult CameraSession::read_ftp_status(Fields& out) {
  if (!connected()) return {};
  auto r = read_props_filtered(*impl_, out, false, true);
#if CRLAB_HAS_RequestFTPTransferResult && CRLAB_HAS_SlotNumber1
  // Asks the camera for its FTP success/failure counters; the answer arrives
  // asynchronously via OnNotifyFTPTransferResult (event sdk.ftp_transfer_result).
  auto err = SDK::RequestFTPTransferResult(impl_->handle, SDK::CrSlotNumber_Slot1);
  out.hex("request_ftp_transfer_result_error", err);
#else
  out.str("request_ftp_transfer_result", "not available in installed SDK");
#endif
  impl_->log.emit("ftp_status.read", Fields().boolean("success", r.ok).hex("sdk_error", r.sdk_error)
                                         .num("duration_ms", r.duration_ms).num("count", r.count));
  return r;
}

}  // namespace crlab
