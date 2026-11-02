// SPDX-License-Identifier: AGPL-3.0-only
#define _GNU_SOURCE
#include "client.h"
#include <ctype.h>
#include <dualsense_usb.h>
#include <math.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "../webos/aurora_filter.inc"
static SDL_GameController *pad;
static dualsense_usb_t *usb;
static SDL_JoystickID instance = -1;
static RpInput state;
static int feedback_enabled, trigger_level = 1, hid_attempted;
static SDL_AudioDeviceID haptic_audio;
static int pulse_status = -999;
static int wired_dualsense_count(void) {
    int count = 0;
    for (int d = 0; d < SDL_NumJoysticks(); d++) {
        SDL_JoystickGUID g = SDL_JoystickGetDeviceGUID(d);
        uint16_t vendor = SDL_JoystickGetDeviceVendor(d),
                 product = SDL_JoystickGetDeviceProduct(d);
        if (g.data[0] == 3 && g.data[1] == 0 && vendor == 0x054c &&
            (product == 0x0ce6 || product == 0x0df2))
            count++;
    }
    return count;
}
static int is_usb_pad(void) {
    if (!pad)
        return 0;
    SDL_JoystickGUID g =
        SDL_JoystickGetGUID(SDL_GameControllerGetJoystick(pad));
    return g.data[0] == 3 && g.data[1] == 0;
}
static int is_usb_dualsense(void) {
    return pad && SDL_GameControllerGetType(pad) == SDL_CONTROLLER_TYPE_PS5 &&
           is_usb_pad();
}
void client_snapshot(RpInput *out) { *out = state; }
void client_release(void) {
    memset(&state, 0, sizeof(state));
    state.connected =
        pad ? (SDL_GameControllerGetType(pad) == SDL_CONTROLLER_TYPE_PS5 ? 2
                                                                         : 1)
            : 0;
    rp_effects_reset();
}
void client_input_close(void) {
    rp_effects_reset();
    if (pad) {
        SDL_GameControllerClose(pad);
        pad = NULL;
    }
    instance = -1;
    memset(&state, 0, sizeof(state));
    hid_attempted = 0;
}
void client_input_scan(void) {
    if (pad && !SDL_GameControllerGetAttached(pad)) {
        client_input_close();
        fprintf(stderr, "Controller disconnected; input released\n");
    }
    if (pad)
        return;
    for (int d = 0; d < SDL_NumJoysticks(); d++) {
        const char *name = SDL_JoystickNameForIndex(d);
        if (webos_name_is_non_gamepad(name) || !SDL_IsGameController(d))
            continue;
        SDL_GameController *candidate = SDL_GameControllerOpen(d);
        if (!candidate)
            continue;
        SDL_Joystick *j = SDL_GameControllerGetJoystick(candidate);
        if (SDL_JoystickNumAxes(j) < 4 || SDL_JoystickNumButtons(j) < 8) {
            SDL_GameControllerClose(candidate);
            continue;
        }
        pad = candidate;
        instance = SDL_JoystickInstanceID(j);
        state.connected =
            SDL_GameControllerGetType(pad) == SDL_CONTROLLER_TYPE_PS5 ? 2 : 1;
        for (int b = 0; b < SDL_CONTROLLER_BUTTON_MAX; b++)
            if (SDL_GameControllerGetButton(pad, b))
                state.buttons |= 1u << b;
        state.lx = SDL_GameControllerGetAxis(pad, SDL_CONTROLLER_AXIS_LEFTX);
        state.ly = SDL_GameControllerGetAxis(pad, SDL_CONTROLLER_AXIS_LEFTY);
        state.rx = SDL_GameControllerGetAxis(pad, SDL_CONTROLLER_AXIS_RIGHTX);
        state.ry = SDL_GameControllerGetAxis(pad, SDL_CONTROLLER_AXIS_RIGHTY);
        state.l2 =
            SDL_GameControllerGetAxis(pad, SDL_CONTROLLER_AXIS_TRIGGERLEFT);
        state.r2 =
            SDL_GameControllerGetAxis(pad, SDL_CONTROLLER_AXIS_TRIGGERRIGHT);
        if (SDL_GameControllerHasSensor(pad, SDL_SENSOR_GYRO))
            SDL_GameControllerSetSensorEnabled(pad, SDL_SENSOR_GYRO, SDL_TRUE);
        if (SDL_GameControllerHasSensor(pad, SDL_SENSOR_ACCEL))
            SDL_GameControllerSetSensorEnabled(pad, SDL_SENSOR_ACCEL, SDL_TRUE);
        fprintf(
            stderr,
            "Controller connected id=%d vendor=%04x product=%04x usb_guid=%d "
            "rumble_advertised=%d\n",
            instance, SDL_GameControllerGetVendor(pad),
            SDL_GameControllerGetProduct(pad), is_usb_pad(),
            SDL_GameControllerHasRumble(pad));
        break;
    }
}
int client_input_event(const SDL_Event *e, RpInput *out) {
    int changed = 0;
    if (e->type == SDL_CONTROLLERDEVICEADDED ||
        e->type == SDL_CONTROLLERDEVICEREMOVED ||
        e->type == SDL_JOYDEVICEADDED || e->type == SDL_JOYDEVICEREMOVED) {
        client_input_scan();
        changed = 1;
    }
    if (pad &&
        (e->type == SDL_CONTROLLERBUTTONDOWN ||
         e->type == SDL_CONTROLLERBUTTONUP) &&
        e->cbutton.which == instance) {
        uint32_t bit = 1u << e->cbutton.button;
        if (e->cbutton.state)
            state.buttons |= bit;
        else
            state.buttons &= ~bit;
        changed = 1;
    }
    if (pad && e->type == SDL_CONTROLLERAXISMOTION &&
        e->caxis.which == instance) {
        switch (e->caxis.axis) {
        case SDL_CONTROLLER_AXIS_LEFTX:
            state.lx = e->caxis.value;
            break;
        case SDL_CONTROLLER_AXIS_LEFTY:
            state.ly = e->caxis.value;
            break;
        case SDL_CONTROLLER_AXIS_RIGHTX:
            state.rx = e->caxis.value;
            break;
        case SDL_CONTROLLER_AXIS_RIGHTY:
            state.ry = e->caxis.value;
            break;
        case SDL_CONTROLLER_AXIS_TRIGGERLEFT:
            state.l2 = e->caxis.value;
            break;
        case SDL_CONTROLLER_AXIS_TRIGGERRIGHT:
            state.r2 = e->caxis.value;
            break;
        default:
            break;
        }
        changed = 1;
    }
    if (pad && e->type == SDL_CONTROLLERSENSORUPDATE &&
        e->csensor.which == instance) {
        for (int i = 0; i < 3; i++) {
            if (e->csensor.sensor == SDL_SENSOR_GYRO)
                state.gyro[i] = (int32_t)(e->csensor.data[i] * 1000.0f);
            else if (e->csensor.sensor == SDL_SENSOR_ACCEL)
                state.accel[i] = (int32_t)(e->csensor.data[i] * 1000.0f /
                                           SDL_STANDARD_GRAVITY);
        }
        state.motion = 1;
        state.timestamp = (uint32_t)(SDL_GetTicks64() * 1000);
        changed = 1;
    }
    if (pad &&
        (e->type == SDL_CONTROLLERTOUCHPADDOWN ||
         e->type == SDL_CONTROLLERTOUCHPADUP ||
         e->type == SDL_CONTROLLERTOUCHPADMOTION) &&
        e->ctouchpad.which == instance && e->ctouchpad.finger == 0 &&
        e->ctouchpad.touchpad == 0) {
        state.touch_active = e->type != SDL_CONTROLLERTOUCHPADUP;
        state.touch_x =
            (uint16_t)(fminf(1.0f, fmaxf(0, e->ctouchpad.x)) * 1919.0f);
        state.touch_y =
            (uint16_t)(fminf(1.0f, fmaxf(0, e->ctouchpad.y)) * 941.0f);
        changed = 1;
    }
    *out = state;
    return changed;
}
int client_nav(const SDL_Event *e) {
    if (e->type != SDL_CONTROLLERBUTTONDOWN || e->cbutton.which != instance)
        return 0;
    switch (e->cbutton.button) {
    case SDL_CONTROLLER_BUTTON_DPAD_UP:
        return 1;
    case SDL_CONTROLLER_BUTTON_DPAD_DOWN:
        return 2;
    case SDL_CONTROLLER_BUTTON_DPAD_LEFT:
        return 3;
    case SDL_CONTROLLER_BUTTON_DPAD_RIGHT:
        return 4;
    case SDL_CONTROLLER_BUTTON_A:
        return 5;
    case SDL_CONTROLLER_BUTTON_B:
        return 6;
    default:
        return 0;
    }
}
void rp_effects_reset(void) {
    if (usb) {
        dualsense_usb_close(usb);
        usb = NULL;
    }
    if (pad)
        SDL_GameControllerRumble(pad, 0, 0, 0);
    rp_haptics_close();
    hid_attempted = 0;
}
void rp_feedback(uint8_t left, uint8_t right, const uint8_t *rgb,
                 const uint8_t *triggers, int enabled, int strength) {
    if (!pad)
        return;
    if (feedback_enabled != enabled || trigger_level != strength) {
        if (usb) {
            dualsense_usb_close(usb);
            usb = NULL;
        }
        feedback_enabled = enabled;
        trigger_level = strength;
        hid_attempted = 0;
    }
    pulse_status = SDL_GameControllerRumble(pad, (uint16_t)left * 257u,
                                            (uint16_t)right * 257u, 100);
    if (enabled && !usb && !hid_attempted) {
        hid_attempted = 1;
        // Upstream fallback can choose another pad: allow it only for one wired
        // DS.
        if (is_usb_dualsense() && wired_dualsense_count() == 1)
            usb = dualsense_usb_open(pad);
    }
    if (!enabled || !usb)
        return;
    uint8_t strength_byte = strength == 0   ? 0xf0
                            : strength == 2 ? 0x60
                            : strength == 3 ? 0x90
                                            : 0;
    if (triggers)
        dualsense_usb_set_strength(usb, strength_byte);
    if (rgb)
        dualsense_usb_set_lightbar(usb, rgb[0], rgb[1], rgb[2]);
    if (triggers) {
        uint8_t off[10] = {0};
        dualsense_usb_set_adaptive_triggers(
            usb, 0x0c, strength ? triggers[0] : 0, strength ? triggers[11] : 0,
            strength ? triggers + 1 : off, strength ? triggers + 12 : off);
    }
}
int rp_haptics_open(const char *name) {
    rp_haptics_close();
    if (!name || !*name || !is_usb_dualsense() || wired_dualsense_count() != 1 ||
        (!strstr(name, "DualSense") && !strstr(name, "Wireless Controller")))
        return -1;
    if (SDL_InitSubSystem(SDL_INIT_AUDIO) < 0)
        return -1;
    SDL_AudioSpec wanted = {0}, obtained = {0};
    wanted.freq = 48000;
    wanted.format = AUDIO_S16LSB;
    wanted.channels = 4;
    wanted.samples = 480;
    haptic_audio = SDL_OpenAudioDevice(name, 0, &wanted, &obtained, 0);
    if (!haptic_audio)
        return -1;
    if (obtained.freq != 48000 || obtained.channels != 4 ||
        obtained.format != AUDIO_S16LSB) {
        rp_haptics_close();
        return -1;
    }
    SDL_PauseAudioDevice(haptic_audio, 0);
    return 0;
}
int rp_haptics_feed(const int16_t *pcm, size_t count) {
    if (!haptic_audio || count != 1920)
        return -1;
    if (SDL_GetQueuedAudioSize(haptic_audio) > 48000 * 4 * 2 / 12)
        SDL_ClearQueuedAudio(haptic_audio);
    return SDL_QueueAudio(haptic_audio, pcm, (Uint32)(count * sizeof(*pcm)));
}
void rp_haptics_close(void) {
    if (haptic_audio) {
        SDL_ClearQueuedAudio(haptic_audio);
        SDL_CloseAudioDevice(haptic_audio);
        haptic_audio = 0;
    }
}
int rp_audio_device_count(void) {
    if (SDL_InitSubSystem(SDL_INIT_AUDIO) < 0)
        return 0;
    int n = SDL_GetNumAudioDevices(0);
    return n > 0 ? n : 0;
}
void rp_audio_device_name(int i, char *out, size_t n) {
    const char *s = SDL_GetAudioDeviceName(i, 0);
    snprintf(out, n, "%s", s ? s : "");
}
int client_input_diagnostics(char *out, size_t n) {
    if (!pad) {
        snprintf(out, n, "Controller: none; USB/BT access unverified");
        return 0;
    }
    snprintf(out, n,
             "%.48s USB:%d HID:%s rumble_api:%d audio4:%d buttons:%08x "
             "axes:%d,%d,%d,%d T:%d,%d",
             SDL_GameControllerName(pad), is_usb_pad(),
             usb ? "open" : "unavailable", pulse_status, haptic_audio != 0,
             state.buttons, state.lx, state.ly, state.rx, state.ry, state.l2,
             state.r2);
    return 1;
}
