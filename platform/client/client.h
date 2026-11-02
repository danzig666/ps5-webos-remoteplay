// SPDX-License-Identifier: AGPL-3.0-only
#pragma once
#include <SDL.h>
#include <stddef.h>
#include <stdint.h>
/* Match the shipped sdl2-compat ABI, not the SDK's webOS SDL2 fork. */
_Static_assert(offsetof(SDL_KeyboardEvent, keysym) == 16, "SDL keyboard ABI mismatch");
_Static_assert(offsetof(SDL_KeyboardEvent, repeat) == 13, "SDL repeat ABI mismatch");
_Static_assert(sizeof(SDL_KeyboardEvent) == 32, "SDL keyboard size mismatch");
/* All functions are called by Rust on the SDL owner thread. */
typedef struct {
  uint32_t buttons;
  int16_t lx, ly, rx, ry, l2, r2;
  uint16_t touch_x, touch_y;
  uint32_t touch_active, connected, motion, timestamp;
  int32_t gyro[3], accel[3];
} RpInput;
typedef struct {
  int32_t kind, value, input_changed;
  char text[64];
  RpInput input;
} RpEvent;
typedef struct {
  int32_t gx, gy, bx, by, rx, ry, wx, wy, max_luma, min_luma, max_cll, max_fall;
} RpHdr;
_Static_assert(sizeof(RpInput) == 60, "Rust/C input layout");
_Static_assert(sizeof(RpEvent) == 136, "Rust/C event layout");
int rp_open(void);
int rp_poll(RpEvent *event);
void rp_begin(int menu);
void rp_text(int row, const char *text, int selected);
void rp_present(void);
void rp_text_input(int active);
void rp_trace_native(uint32_t phase);
void rp_close(void);
int rp_media_open(int width, int height, int fps, int codec, int rate,
                  int channels, int samples, const RpHdr *hdr);
int rp_video(const uint8_t *data, size_t size, int key);
int rp_audio(const uint8_t *data, size_t size, int muted);
void rp_media_close(void);
void rp_feedback(uint8_t left, uint8_t right, const uint8_t *rgb,
                 const uint8_t *triggers, int enabled, int trigger_strength);
void rp_effects_reset(void);
int rp_haptics_open(const char *name);
int rp_haptics_feed(const int16_t *pcm, size_t samples);
void rp_haptics_close(void);
int rp_audio_device_count(void);
void rp_audio_device_name(int index, char *out, size_t size);
int rp_diagnostics(char *out, size_t size);
int rp_run(void);
