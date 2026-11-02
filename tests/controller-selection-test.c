/* Host-only selection test: no device access or physical output. */
#include "../platform/client/input.c"
#include <assert.h>
static SDL_GameControllerType test_type;
static Uint8 test_bus;
SDL_GameControllerType SDL_GameControllerGetType(SDL_GameController *p) {
    (void)p;
    return test_type;
}
SDL_Joystick *SDL_GameControllerGetJoystick(SDL_GameController *p) {
    return (SDL_Joystick *)p;
}
SDL_JoystickGUID SDL_JoystickGetGUID(SDL_Joystick *j) {
    (void)j;
    SDL_JoystickGUID g = {{0}};
    g.data[0] = test_bus;
    return g;
}
int main(void) {
    int dummy;
    pad = (SDL_GameController *)&dummy;
    test_bus = 3;
    test_type = SDL_CONTROLLER_TYPE_XBOXONE;
    assert(!is_usb_dualsense());
    test_type = SDL_CONTROLLER_TYPE_PS5;
    assert(is_usb_dualsense());
    test_bus = 5;
    assert(!is_usb_dualsense());
    pad = NULL;
    assert(!is_usb_dualsense());
    return 0;
}
