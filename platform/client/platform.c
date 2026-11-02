// SPDX-License-Identifier: AGPL-3.0-only
#define _GNU_SOURCE
#include "client.h"
#include <SDL_test_font.h>
#include <SDL_webOS.h>
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
  if (active)
    SDL_StartTextInput();
  else
    SDL_StopTextInput();
  text_input_active = active;
}
int rp_poll(RpEvent *out) {
  memset(out, 0, sizeof(*out));
  if (stopping) {
    out->kind = 9;
    return 1;
  }
  if (SDL_GetTicks() - scan_time > 2000) {
    client_input_scan();
    scan_time = SDL_GetTicks();
    out->input_changed = 1;
    client_snapshot(&out->input);
    return 1;
  }
  SDL_Event e;
  if (!SDL_PollEvent(&e))
    return 0;
  out->input_changed = client_input_event(&e, &out->input);
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
    switch (e.key.keysym.sym) {
    case SDLK_UP:
      out->kind = 1;
      break;
    case SDLK_DOWN:
      out->kind = 2;
      break;
    case SDLK_LEFT:
      out->kind = 3;
      break;
    case SDLK_RIGHT:
      out->kind = 4;
      break;
    case SDLK_RETURN:
    case SDLK_KP_ENTER:
      out->kind = 5;
      break;
    case SDLK_ESCAPE:
      out->kind = 6;
      out->value = 1;
      break;
    case SDLK_BACKSPACE:
      out->kind = 8;
      break;
    case SDLK_F1:
      out->kind = 11;
      break;
    default:
      break;
    }
    if (e.key.keysym.scancode == SDL_WEBOS_SCANCODE_BACK) {
      out->kind = 6;
      out->value = 1;
    }
    if (e.key.keysym.scancode == SDL_WEBOS_SCANCODE_EXIT)
      out->kind = 9;
    if (e.key.keysym.scancode == SDL_WEBOS_SCANCODE_RED)
      out->kind = 11;
  } else {
    out->kind = client_nav(&e);
  }
  return 1;
}
void rp_begin(int menu) {
  SDL_SetRenderDrawColor(renderer, 0, 0, 0, 0);
  SDL_RenderClear(renderer);
  SDL_SetRenderDrawColor(renderer, 12, 22, 32, 255);
  SDL_Rect p = {0, 0, 1280, menu ? 700 : 62};
  SDL_RenderFillRect(renderer, &p);
}
void rp_text(int row, const char *s, int selected) {
  int y = 8 + row * 19;
  if (selected) {
    SDL_SetRenderDrawColor(renderer, 35, 85, 115, 255);
    SDL_Rect p = {5, y - 3, 1240, 18};
    SDL_RenderFillRect(renderer, &p);
  }
  SDL_SetRenderDrawColor(renderer, 245, 245, 245, 255);
  SDLTest_DrawString(renderer, 12, y, s);
}
void rp_present(void) {
  static int first = 1;
  if (first)
    fprintf(stderr, "Startup presenting first UI frame\n");
  SDL_RenderPresent(renderer);
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
