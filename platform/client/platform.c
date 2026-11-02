// SPDX-License-Identifier: AGPL-3.0-only
#define _GNU_SOURCE
#include "client.h"
#include <SDL_test_font.h>
#include "../../upstream/aurora-tv/src/app/platform/webos/sdl_webos_scancode.h"
#include "navigation.h"
#include <opus.h>
#include <signal.h>
#include <ss4s.h>
#include <stdarg.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
static SDL_Window *window;
static SDL_Renderer *renderer;
static SS4S_Player *player;
static int ss_ready, video_open, audio_open;
static uint8_t silence[4096];
static int silence_size;
static volatile sig_atomic_t stopping;
extern void probe_devices(void);
extern void client_input_scan(void), client_input_close(void),
    client_snapshot(RpInput *), client_release(void);
extern int client_input_event(const SDL_Event *, RpInput *),
    client_nav(const SDL_Event *), client_input_diagnostics(char *, size_t);
static Uint32 scan_time;
static int text_input_active;
static unsigned logged_render_errors;
static void render_result(const char *operation, int result) {
  if (result < 0 && logged_render_errors++ < 16)
    fprintf(stderr, "Render failure operation=%s error=%s\n", operation, SDL_GetError());
  rp_trace_native(0);
}
static int startup_failed(const char *stage) {
  fprintf(stderr, "Startup FAILED stage=%s SDL_error=%s\n", stage, SDL_GetError());
  return -1;
}
static void stop_signal(int sig) {
  (void)sig;
  stopping = 1;
}
static void ss_log(SS4S_LogLevel level, const char *tag, const char *fmt, ...) {
  if (level > SS4S_LogLevelInfo)
    return;
  fprintf(stderr, "SS4S t=%u level=%d %s ", SDL_GetTicks(), level, tag);
  va_list a;
  va_start(a, fmt);
  vfprintf(stderr, fmt, a);
  va_end(a);
  fputc('\n', stderr);
}
int rp_open(void) {
  signal(SIGINT, stop_signal);
  signal(SIGTERM, stop_signal);
  setenv("APPID", "org.homebrew.ps5remote", 1);
  SDL_SetHint("SDL_WEBOS_REGISTER_APP", "1");
  SDL_SetHint("SDL_WEBOS_ACCESS_POLICY_KEYS_BACK", "1");
  SDL_SetHint("SDL_WEBOS_ACCESS_POLICY_KEYS_EXIT", "1");
    SDL_SetHint("SDL_WEBOS_ACCESS_POLICY_KEYS_GUIDE", "1");
  fprintf(stderr, "Startup begin SDL video/controller init\n");
  if (SDL_Init(SDL_INIT_VIDEO | SDL_INIT_GAMECONTROLLER | SDL_INIT_HAPTIC) < 0)
    return startup_failed("SDL_Init");
  fprintf(stderr, "Startup SDL initialized; creating webOS window\n");
  SDL_GL_SetAttribute(SDL_GL_ALPHA_SIZE, 8);
  SDL_GL_SetAttribute(SDL_GL_RED_SIZE, 8);
  SDL_GL_SetAttribute(SDL_GL_GREEN_SIZE, 8);
  SDL_GL_SetAttribute(SDL_GL_BLUE_SIZE, 8);
  window = SDL_CreateWindow("PS5 Remote", SDL_WINDOWPOS_UNDEFINED,
                            SDL_WINDOWPOS_UNDEFINED, 1920, 1080,
                            /* Match Aurora: the webOS shell stretches this
                             * surface. SDL fullscreen invokes an unsupported
                             * toplevel/exclusive modeset through sdl2-compat. */
                            SDL_WINDOW_OPENGL | SDL_WINDOW_SHOWN);
  if (!window)
    return startup_failed("SDL_CreateWindow");
  /* sdl2-compat may start webOS IME automatically during window creation. */
  SDL_StopTextInput();
  text_input_active = 0;
  fprintf(stderr, "Startup window created; IME stopped; creating renderer\n");
  renderer = SDL_CreateRenderer(window, -1, SDL_RENDERER_ACCELERATED);
  if (!renderer)
    return startup_failed("SDL_CreateRenderer");
  SDL_RenderSetLogicalSize(renderer, 1280, 720);
  int alpha = 0;
  SDL_GL_GetAttribute(SDL_GL_ALPHA_SIZE, &alpha);
  if (alpha < 8) {
    fprintf(stderr, "Startup alpha_bits=%d (requires 8)\n", alpha);
    return startup_failed("EGL alpha");
  }
  fprintf(stderr, "SDL renderer=%s alpha=%d\n", SDL_GetCurrentVideoDriver(),
          alpha);
  SDL_RendererInfo info = {0};
  SDL_version runtime;
  SDL_GetVersion(&runtime);
  int output_w = 0, output_h = 0, window_w = 0, window_h = 0;
  SDL_GetRendererInfo(renderer, &info);
  SDL_GetRendererOutputSize(renderer, &output_w, &output_h);
  SDL_GetWindowSize(window, &window_w, &window_h);
  fprintf(stderr, "Graphics SDL=%u.%u.%u renderer=%s flags=%08x window=%dx%d output=%dx%d window_flags=%08x keyboard_focus=%d\n",
          runtime.major, runtime.minor, runtime.patch, info.name ? info.name : "unknown",
          info.flags, window_w, window_h, output_w, output_h, SDL_GetWindowFlags(window),
          SDL_GetKeyboardFocus() == window);
  SDL_GameControllerAddMappingsFromFile("assets/gamecontrollerdb.txt");
  SDL_GameControllerAddMappingsFromFile("assets/gamecontrollerdb_extra.txt");
  fprintf(stderr, "Startup probing input devices\n");
  probe_devices();
  fprintf(stderr, "Startup scanning controllers\n");
  client_input_scan();
  fprintf(stderr, "Startup loading SS4S modules (NDL init deferred until playback)\n");
  SS4S_Config config = {.audioDriver = "ndl-webos5",
                        .videoDriver = "ndl-webos5",
                        .loggingFunction = ss_log};
  if (SS4S_Init(0, NULL, &config) != 0)
    return -1;
  ss_ready = 1;
  if (SS4S_PostInit(0, NULL) != 0)
    return -1;
  fprintf(stderr, "Startup platform ready\n");
  return 0;
}
void rp_text_input(int active) {
  active = active != 0;
  if (active == text_input_active)
    return;
  rp_trace_native(8);
  if (active)
    SDL_StartTextInput();
  else
    SDL_StopTextInput();
  text_input_active = active;
  rp_trace_native(0);
}
int rp_poll(RpEvent *out) {
  memset(out, 0, sizeof(*out));
  if (stopping) {
    out->kind = 9;
    return 1;
  }
  if (SDL_GetTicks() - scan_time > 2000) {
    rp_trace_native(1);
    client_input_scan();
    rp_trace_native(0);
    scan_time = SDL_GetTicks();
    out->input_changed = 1;
    client_snapshot(&out->input);
    return 1;
  }
  SDL_Event e;
  rp_trace_native(2);
  int available = SDL_PollEvent(&e);
  rp_trace_native(0);
  if (!available)
    return 0;
  static unsigned diagnostic_events;
  if (diagnostic_events < 128 && e.type != SDL_MOUSEMOTION && e.type != SDL_CONTROLLERSENSORUPDATE) {
    fprintf(stderr, "Input t=%u event_type=%u window_event=%u window_flags=%08x keyboard_focus=%d\n",
            SDL_GetTicks(), e.type, e.type == SDL_WINDOWEVENT ? e.window.event : 0,
            SDL_GetWindowFlags(window), SDL_GetKeyboardFocus() == window);
    diagnostic_events++;
  }
  rp_trace_native(3);
  out->input_changed = client_input_event(&e, &out->input);
  rp_trace_native(0);
  if (e.type == SDL_QUIT) {
    out->kind = 9;
    return 1;
  }
  if (e.type == SDL_APP_WILLENTERBACKGROUND ||
      (e.type == SDL_WINDOWEVENT &&
       (e.window.event == SDL_WINDOWEVENT_FOCUS_LOST ||
        e.window.event == SDL_WINDOWEVENT_MINIMIZED))) {
    client_release();
    client_snapshot(&out->input);
    out->input_changed = 1;
    out->kind = 10;
    return 1;
  }
  if (e.type == SDL_TEXTINPUT) {
    out->kind = 7;
    snprintf(out->text, sizeof(out->text), "%s", e.text.text);
    return 1;
  }
  if (e.type == SDL_KEYDOWN) {
    client_key_event(&e.key, out);
    static unsigned logged_keys;
    if (logged_keys++ < 64)
      fprintf(stderr, "Keyboard navigation=%d repeat=%u (character values omitted)\n", out->kind, e.key.repeat);
  } else {
    out->kind = client_nav(&e);
  }
  return 1;
}
void rp_begin(int menu) {
  rp_trace_native(4);
  SDL_SetRenderDrawColor(renderer, 0, 0, 0, 0);
  render_result("clear", SDL_RenderClear(renderer));
  rp_trace_native(5);
  SDL_SetRenderDrawColor(renderer, 12, 22, 32, 255);
  SDL_Rect p = {0, 0, 1280, menu ? 700 : 62};
  render_result("panel", SDL_RenderFillRect(renderer, &p));
}
void rp_text(int row, const char *s, int selected) {
  rp_trace_native(5);
  int y = 8 + row * 19;
  if (selected) {
    SDL_SetRenderDrawColor(renderer, 35, 85, 115, 255);
    SDL_Rect p = {5, y - 3, 1240, 18};
    render_result("selection", SDL_RenderFillRect(renderer, &p));
  }
  SDL_SetRenderDrawColor(renderer, 245, 245, 245, 255);
  rp_trace_native(6);
  render_result("text", SDLTest_DrawString(renderer, 12, y, s));
}
void rp_present(void) {
  static int first = 1;
  if (first)
    fprintf(stderr, "Startup presenting first UI frame\n");
  rp_trace_native(7);
  SDL_ClearError();
  SDL_RenderPresent(renderer);
  if (*SDL_GetError())
    render_result("present", -1);
  rp_trace_native(0);
  if (first) {
    fprintf(stderr, "Startup first UI frame submitted; event loop running\n");
    first = 0;
  }
}
void rp_media_close(void) {
  if (audio_open) {
    SS4S_PlayerAudioClose(player);
    audio_open = 0;
  }
  if (video_open) {
    SS4S_PlayerVideoSetHDRInfo(player, NULL);
    SS4S_PlayerVideoClose(player);
    video_open = 0;
  }
  if (player) {
    SS4S_PlayerClose(player);
    player = NULL;
  }
  silence_size = 0;
}
int rp_media_open(int width, int height, int fps, int codec, int rate,
                  int channels, int samples, const RpHdr *hdr) {
  rp_media_close();
  SS4S_VideoCapabilities caps = {0};
  if (!SS4S_GetVideoCapabilities(&caps))
    return -2;
  SS4S_VideoCodec vc = codec ? SS4S_VIDEO_H265 : SS4S_VIDEO_H264;
  if (!(caps.codecs & vc))
    return -2;
  if (codec == 2 && (!caps.hdr || !hdr))
    return -3;
  if (width < 1 || width > 1920 || height < 1 || height > 1080 ||
      channels < 1 || channels > 2 || samples < 120 || samples > 1920)
    return -1;
  player = SS4S_PlayerOpen();
  if (!player)
    return -1;
  SS4S_PlayerSetWaitAudioVideoReady(player, true);
  SS4S_PlayerSetViewportSize(player, 1920, 1080);
  SS4S_AudioInfo ai = {.codec = SS4S_AUDIO_OPUS,
                       .sampleRate = rate,
                       .numOfChannels = channels,
                       .samplesPerFrame = samples,
                       .appName = "org.homebrew.ps5remote",
                       .streamName = "PS5 Remote"};
  int rc = SS4S_PlayerAudioOpen(player, &ai);
  if (rc != 0) {
    rp_media_close();
    return -1;
  }
  audio_open = 1;
  SS4S_VideoInfo vi = {.codec = vc,
                       .width = width,
                       .height = height,
                       .frameRateNumerator = fps,
                       .frameRateDenominator = 1};
  rc = SS4S_PlayerVideoOpen(player, &vi);
  if (rc != 0) {
    rp_media_close();
    return -2;
  }
  video_open = 1;
  if (codec == 2) {
    SS4S_VideoHDRInfo hi = {.displayPrimariesX = {hdr->rx, hdr->gx, hdr->bx},
                            .displayPrimariesY = {hdr->ry, hdr->gy, hdr->by},
                            .whitePointX = hdr->wx,
                            .whitePointY = hdr->wy,
                            .maxDisplayMasteringLuminance = hdr->max_luma,
                            .minDisplayMasteringLuminance = hdr->min_luma,
                            .maxContentLightLevel = hdr->max_cll,
                            .maxPicAverageLightLevel = hdr->max_fall,
                            .transferCharacteristics = 16,
                            .colorPrimaries = 9,
                            .matrixCoefficients = 9,
                            .videoFullRange = 0};
    if (!SS4S_PlayerVideoSetHDRInfo(player, &hi)) {
      rp_media_close();
      return -3;
    }
  }
  int err = 0;
  OpusEncoder *enc =
      opus_encoder_create(rate, channels, OPUS_APPLICATION_AUDIO, &err);
  if (enc) {
    int16_t zeros[3840] = {0};
    silence_size = opus_encode(enc, zeros, samples, silence, sizeof(silence));
    opus_encoder_destroy(enc);
  }
  fprintf(stderr,
          "Media opened %dx%d fps=%d codec=%d rate=%d channels=%d "
          "hardware_output=UNVERIFIED\n",
          width, height, fps, codec, rate, channels);
  return 0;
}
int rp_video(const uint8_t *d, size_t n, int key) {
  if (!video_open)
    return -1;
  return SS4S_PlayerVideoFeed(player, d, n,
                              SS4S_VIDEO_FEED_DATA_FRAME_START |
                                  SS4S_VIDEO_FEED_DATA_FRAME_END |
                                  (key ? SS4S_VIDEO_FEED_DATA_KEYFRAME : 0));
}
int rp_audio(const uint8_t *d, size_t n, int muted) {
  if (!audio_open)
    return -1;
  if (muted) {
    if (silence_size <= 0)
      return -1;
    d = silence;
    n = (size_t)silence_size;
  }
  return SS4S_PlayerAudioFeed(player, d, n);
}
int rp_diagnostics(char *out, size_t n) {
  return client_input_diagnostics(out, n);
}
void rp_close(void) {
  rp_effects_reset();
  rp_haptics_close();
  client_input_close();
  rp_media_close();
  if (ss_ready) {
    SS4S_Quit();
    ss_ready = 0;
  }
  if (renderer) {
    SDLTest_CleanupTextDrawing();
    SDL_DestroyRenderer(renderer);
        renderer = NULL;
  }
    if (window) {
    SDL_DestroyWindow(window);
        window = NULL;
    }
  SDL_Quit();
  fprintf(stderr, "Client stopped\n");
}
