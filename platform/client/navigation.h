#pragma once
#include "client.h"
#include "../../upstream/aurora-tv/src/app/platform/webos/sdl_webos_scancode.h"
/* The shipped SDL3/compat runtime uses Aurora's codes, not SDK SDL2 codes. */
static inline void client_key_event(const SDL_KeyboardEvent *key, RpEvent *out) {
    switch ((int)key->keysym.scancode) {
    case SDL_SCANCODE_UP: out->kind = 1; break;
    case SDL_SCANCODE_DOWN: out->kind = 2; break;
    case SDL_SCANCODE_LEFT: out->kind = 3; break;
    case SDL_SCANCODE_RIGHT: out->kind = 4; break;
    case SDL_SCANCODE_RETURN:
    case SDL_SCANCODE_KP_ENTER: out->kind = 5; break;
    case SDL_SCANCODE_ESCAPE:
    case SDL_SCANCODE_WEBOS_BACK: out->kind = 6; out->value = 1; break;
    case SDL_SCANCODE_BACKSPACE: out->kind = 8; break;
    case SDL_SCANCODE_WEBOS_EXIT: out->kind = 9; break;
    case SDL_SCANCODE_F1:
    case SDL_SCANCODE_WEBOS_RED: out->kind = 11; break;
    default: break;
    }
    if (out->kind) return;
    switch (key->keysym.sym) {
    case SDLK_UP: out->kind = 1; break;
    case SDLK_DOWN: out->kind = 2; break;
    case SDLK_LEFT: out->kind = 3; break;
    case SDLK_RIGHT: out->kind = 4; break;
    case SDLK_RETURN:
    case SDLK_KP_ENTER: out->kind = 5; break;
    case SDLK_ESCAPE: out->kind = 6; out->value = 1; break;
    case SDLK_BACKSPACE: out->kind = 8; break;
    case SDLK_F1: out->kind = 11; break;
    default: break;
    }
}
