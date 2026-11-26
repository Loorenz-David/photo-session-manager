#include <chrono>
#include <thread>
#include <mutex>
#include <vector>
#include "mini_test.hpp"
#include "crlab/liveview.hpp"

using namespace crlab;
using namespace std::chrono_literals;

TEST_CASE("lv1_c1a_first_frame_paced") {
  LivePacer p; p.reset(1); auto r=p.on_pull(PullKind::Ok,7,{1},0,1000);
  CHECK(r.forward); CHECK_EQ(r.next_ms,1030);
}
TEST_CASE("lv1_c1b_numbered_repeat") {
  LivePacer p; p.reset(1); p.on_pull(PullKind::Ok,7,{1},0,1000);
  auto r=p.on_pull(PullKind::Ok,7,{2},0,1031);
  CHECK(!r.forward); CHECK_EQ(p.repeats(),1u); CHECK_EQ(r.next_ms,1036);
}
TEST_CASE("lv1_c1c_unnumbered_changed") {
  LivePacer p; p.reset(1); CHECK(p.on_pull(PullKind::Ok,0,{'A','B'},0,10).forward);
  CHECK(p.on_pull(PullKind::Ok,0,{'A','C'},0,20).forward);
}
TEST_CASE("lv1_c1d_unnumbered_repeat") {
  LivePacer p; p.reset(1); p.on_pull(PullKind::Ok,0,{'A','B'},0,10);
  auto r=p.on_pull(PullKind::Ok,0,{'A','B'},0,20); CHECK(!r.forward); CHECK_EQ(r.next_ms,25);
}
TEST_CASE("lv1_c1e_not_updated") {
  LivePacer p; p.reset(1); auto r=p.on_pull(PullKind::NotUpdated,0,{},0,10);
  CHECK(!r.forward); CHECK_EQ(r.next_ms,15); CHECK_EQ(p.errors(),0u);
}
TEST_CASE("lv1_c1f_empty_first_pull") {
  LivePacer p; p.reset(1); auto r=p.on_pull(PullKind::Error,0,{},0,10);
  CHECK(!r.forward); CHECK_EQ(r.next_ms,30); CHECK_EQ(p.errors(),0u);
}
TEST_CASE("lv1_c1g_sdk_error") {
  LivePacer p; p.reset(1); auto r=p.on_pull(PullKind::Error,0,{},0x8000,10);
  CHECK(!r.forward); CHECK_EQ(r.next_ms,30); CHECK_EQ(p.errors(),1u);
}
TEST_CASE("lv1_c1h_median_clamp") {
  LivePacer p; p.reset(1); int64_t t=0; PaceResult r;
  for (uint32_t i=1;i<=17;++i) { t+=50; r=p.on_pull(PullKind::Ok,i,{1},0,t); }
  CHECK_EQ(r.next_ms,t+40);
  for (uint32_t i=18;i<=34;++i) { t+=300; r=p.on_pull(PullKind::Ok,i,{1},0,t); }
  CHECK_EQ(r.next_ms,t+190);
}
TEST_CASE("lv1_c2a_epoch_reset") {
  LivePacer p; p.reset(1); p.on_pull(PullKind::Ok,900,{1},0,1); p.reset(2);
  CHECK(p.on_pull(PullKind::Ok,1,{1},0,2).forward);
}
TEST_CASE("lv1_c3a_record_bytes") {
  FrameRecord r; r.epoch=2; r.frame_no=7; r.width=1024; r.height=680; r.jpeg={1,2,3,4,5};
  std::vector<uint8_t> expected={'P','S','L','V',1,1,0,0,2,0,0,0,7,0,0,0,0,4,0xa8,2,5,0,0,0,1,2,3,4,5};
  CHECK_EQ(r.encode(),expected);
}
TEST_CASE("lv1_c3b_record_size_bounds") {
  FrameRecord r; CHECK(r.encode().empty()); r.jpeg.resize(4194305); CHECK(r.encode().empty());
}
TEST_CASE("lv1_c3c_jpeg_dimensions") {
  std::vector<uint8_t> jpeg={0xff,0xd8,0xff,0xc0,0,11,8,1,0xa8,2,0x80,3,1,0x11,0,2,0x11,0,3,0x11,0,0xff,0xd9};
  CHECK_EQ(FrameRecord::jpeg_size(jpeg),std::make_pair(uint16_t(640),uint16_t(424)));
  CHECK_EQ(FrameRecord::jpeg_size(std::vector<uint8_t>(10,7)),std::make_pair(uint16_t(0),uint16_t(0)));
}
TEST_CASE("lv1_c4a_latest_slot") {
  LatestSlot<int> s; s.put(1); s.put(2); CHECK_EQ(s.take(0ms),std::optional<int>(2)); CHECK(!s.take(0ms));
}
TEST_CASE("lv1_c4b_slow_consumer") {
  LatestSlot<int> s; s.put(1); auto held=s.take(0ms); CHECK_EQ(held,std::optional<int>(1));
  auto start=std::chrono::steady_clock::now(); s.put(2); s.put(3); s.put(4);
  CHECK(std::chrono::steady_clock::now()-start < 50ms); CHECK_EQ(s.take(0ms),std::optional<int>(4));
}
TEST_CASE("lv1_c5a_focus_supersedes_waiting") {
  FocusQueue q; q.push({1,1,true,"B"}); auto old=q.push({2,2,true,"C"});
  CHECK(old); CHECK_EQ(old->cmd_id,"B"); CHECK_EQ(q.take(0ms)->cmd_id,"C");
}
TEST_CASE("lv1_c5b_focus_parse") {
  CHECK(!parse_focus_point("640","10",true,"k")); CHECK(!parse_focus_point("","10",true,"k"));
  auto j=parse_focus_point("320","240",true,"k"); CHECK(j); CHECK_EQ(j->x,320); CHECK_EQ(j->y,240);
  CHECK(j->af); CHECK_EQ(j->cmd_id,"k");
}
TEST_CASE("lv1_c5c_action_lock") {
  ActionLock action; std::vector<int> order; auto start=std::chrono::steady_clock::now();
  std::thread focus([&]{ action.lock_focus(); order.push_back(1); std::this_thread::sleep_for(200ms); order.push_back(2); action.unlock(); });
  std::this_thread::sleep_for(50ms);
  std::thread capture([&]{ action.lock_capture(); order.push_back(3); std::this_thread::sleep_for(40ms); order.push_back(4); action.unlock(); });
  std::this_thread::sleep_for(20ms);
  std::thread focus2([&]{ action.lock_focus(); order.push_back(5); action.unlock(); });
  focus.join(); capture.join(); focus2.join();
  CHECK_EQ(order.front(),1); CHECK_EQ(order[1],2); CHECK_EQ(order.size(),5u);
  // The main-loop capture has priority over a focus queued while it is waiting.
  CHECK_EQ(order[2],3); CHECK_EQ(order[3],4); CHECK_EQ(order[4],5);
  CHECK(std::chrono::steady_clock::now()-start >= 200ms);
}

int main() { return mt_run_all(); }
