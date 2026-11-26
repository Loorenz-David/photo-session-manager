#include <atomic>
#include <cerrno>
#include <chrono>
#include <poll.h>
#include <sys/socket.h>
#include <thread>
#include <utility>

#include "crlab/camera.hpp"
#include "crlab/liveview.hpp"

namespace crlab {

struct LiveRunner::Impl {
  CameraSession& session; EventSink& log; ActionLock& action; int fd;
  std::atomic<bool> stop{true};
  bool streaming=false;
  uint32_t epoch=0;
  LivePacer pacer;
  LatestSlot<std::vector<uint8_t>> slot;
  FocusQueue focus_queue;
  std::thread pull_thread, sender_thread, focus_thread;
  Impl(CameraSession& s, EventSink& l, ActionLock& a, int f):session(s),log(l),action(a),fd(f){}
  bool write_all(const std::vector<uint8_t>& bytes) {
    size_t off=0;
    while (off<bytes.size() && !stop) {
      pollfd p{fd,POLLOUT,0};
      if (poll(&p,1,30)<=0) continue;
      ssize_t n=send(fd,bytes.data()+off,bytes.size()-off,MSG_NOSIGNAL|MSG_DONTWAIT);
      if (n>0) off+=static_cast<size_t>(n);
      else if (n<0 && errno!=EAGAIN && errno!=EINTR) return false;
    }
    return off==bytes.size();
  }
  void pull() {
    int64_t next=monotonic_ms(); int64_t stats_at=next;
    uint64_t prev_frames=0,prev_repeats=0,prev_errors=0;
    while (!stop) {
      int64_t now=monotonic_ms();
      if (now<next) { std::this_thread::sleep_for(std::chrono::milliseconds(std::min<int64_t>(5,next-now))); continue; }
      auto frame=session.pull_live_view_frame();
      PullKind kind=frame.status==LvPull::Ok?PullKind::Ok:frame.status==LvPull::NotUpdated?PullKind::NotUpdated:
                    frame.status==LvPull::Unsupported?PullKind::Unsupported:PullKind::Error;
      now=monotonic_ms(); auto decision=pacer.on_pull(kind,frame.frame_no,frame.jpeg,frame.sdk_error,now);
      next=decision.next_ms;
      if (decision.stop) { log.emit("liveview.unavailable",Fields().str("reason","sdk")); break; }
      if (decision.forward) {
        auto dims=FrameRecord::jpeg_size(frame.jpeg);
        FrameRecord record{epoch,frame.frame_no,dims.first,dims.second,std::move(frame.jpeg)};
        auto bytes=record.encode(); if (!bytes.empty()) slot.put(std::move(bytes));
      }
      if (now-stats_at>=10000) {
        uint64_t f=pacer.frames(),r=pacer.repeats(),e=pacer.errors();
        log.emit("liveview.stats",Fields().unum("epoch",epoch).unum("frames",f).unum("repeats",r)
                 .unum("errors",e).dbl("fps",static_cast<double>(f-prev_frames)*1000.0/(now-stats_at)));
        prev_frames=f;prev_repeats=r;prev_errors=e;stats_at=now;
        (void)prev_repeats;(void)prev_errors;
      }
    }
  }
  void sender() {
    while (!stop) {
      auto b=slot.take(std::chrono::milliseconds(30));
      if (b && !write_all(*b)) { log.emit("liveview.unavailable",Fields().str("reason","frame_channel")); break; }
    }
  }
  void emit_focus_frames() {
    bool ok=false; auto frames=session.focus_frames(&ok); if (!ok) return;
    std::string json="[";
    for (const auto& f:frames) {
      if (json.size()>1) json+=",";
      json+="["+std::to_string(f.type)+","+std::to_string(f.state)+","+
            std::to_string(f.x_num)+","+std::to_string(f.x_den)+","+
            std::to_string(f.y_num)+","+std::to_string(f.y_den)+","+
            std::to_string(f.width)+","+std::to_string(f.height)+"]";
    }
    json+="]";
    if (json!=last_focus_frames) { last_focus_frames=json; log.emit("liveview.focus_frame",Fields().raw("frames",json)); }
  }
  void emit_exposure() {
    Fields props; if (!session.read_properties(props,false).ok) return;
    Fields out;
    for (const auto& kv:props.items()) if (kv.first=="shutter_speed"||kv.first=="f_number"||kv.first=="iso") out.raw(kv.first,kv.second);
    auto json=out.encode_object();
    if (json!=last_exposure) { last_exposure=json; log.emit("liveview.exposure",out); }
  }
  std::string last_focus_frames,last_exposure;
  void focus() {
    int64_t focus_at=monotonic_ms()-250,exposure_at=monotonic_ms()-1000;
    while (!stop) {
      auto job=focus_queue.take(std::chrono::milliseconds(25));
      if (stop) break;
      if (job) {
        action.lock_focus();
        int64_t start=monotonic_ms(),point_ms=0,af_ms=0; uint32_t err=0;
        auto state=session.focus_state();
        if (!state.af_position_writable) {
          log.emit("focus.result",Fields().str("cmd_id",job->cmd_id).num("x",job->x).num("y",job->y)
                   .str("af","skipped").boolean("point_set",false).str("refused","focus_area_not_movable")
                   .num("point_ms",0).num("af_ms",0).num("total_ms",monotonic_ms()-start));
        } else {
          bool point=session.set_focus_point(job->x,job->y,&point_ms,&err);
          std::string af="skipped";
          if (point && job->af && !stop) af=session.autofocus(3000,&af_ms,&err);
          log.emit("focus.result",Fields().str("cmd_id",job->cmd_id).num("x",job->x).num("y",job->y)
                   .str("af",af).boolean("point_set",point).num("point_ms",point_ms)
                   .num("af_ms",af_ms).num("total_ms",monotonic_ms()-start));
        }
        action.unlock();
      }
      int64_t now=monotonic_ms();
      if (now-focus_at>=250) { emit_focus_frames(); focus_at=now; }
      if (now-exposure_at>=1000) { emit_exposure(); exposure_at=now; }
    }
  }
};

LiveRunner::LiveRunner(CameraSession& s,EventSink& l,ActionLock& a,int fd):impl_(std::make_unique<Impl>(s,l,a,fd)){}
LiveRunner::~LiveRunner(){stop("shutdown");}
bool LiveRunner::running() const {return !impl_->stop;}
void LiveRunner::start(uint32_t epoch) {
  if (running()) return;
  impl_->epoch=epoch; impl_->session.resume_autofocus(); impl_->stop=false;
  impl_->focus_thread=std::thread([this]{impl_->focus();});
  auto state=impl_->session.live_view_state();
  if (state.status!=2) {
    impl_->log.emit("liveview.unavailable",Fields().str("reason",state.status==0?"not_supported":"disabled"));return;
  }
  if (impl_->fd<0) {impl_->log.emit("liveview.unavailable",Fields().str("reason","frame_channel"));return;}
  impl_->pacer.reset(epoch); impl_->streaming=true;
  impl_->log.emit("liveview.started",Fields().unum("epoch",epoch).str("transport",to_string(impl_->session.transport()))
                 .str("status","enabled").str("frame_channel","connected"));
  impl_->pull_thread=std::thread([this]{impl_->pull();});
  impl_->sender_thread=std::thread([this]{impl_->sender();});
}
void LiveRunner::stop(const std::string& reason) {
  if (!running()) return;
  impl_->stop=true; impl_->session.cancel_autofocus(); impl_->focus_queue.close(); impl_->slot.close();
  if (impl_->pull_thread.joinable()) impl_->pull_thread.join();
  if (impl_->sender_thread.joinable()) impl_->sender_thread.join();
  if (impl_->focus_thread.joinable()) impl_->focus_thread.join();
  if (impl_->streaming) impl_->log.emit("liveview.stopped",Fields().unum("epoch",impl_->epoch).str("reason",reason)
                                  .unum("frames",impl_->pacer.frames()).unum("repeats",impl_->pacer.repeats())
                                  .unum("errors",impl_->pacer.errors()));
}
void LiveRunner::queue_focus(FocusJob job) {
  auto old=impl_->focus_queue.push(std::move(job));
  if (old) impl_->log.emit("focus.result",Fields().str("cmd_id",old->cmd_id).str("af","superseded")
                          .boolean("point_set",false));
}

} // namespace crlab
