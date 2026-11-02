#include "../platform/client/navigation.h"
#include <assert.h>
#include <string.h>
int main(void) {
    const int scans[] = {SDL_SCANCODE_UP, SDL_SCANCODE_DOWN, SDL_SCANCODE_LEFT,
        SDL_SCANCODE_RIGHT, SDL_SCANCODE_RETURN, SDL_SCANCODE_WEBOS_BACK,
        SDL_SCANCODE_WEBOS_EXIT, SDL_SCANCODE_WEBOS_RED};
    const int expected[] = {1, 2, 3, 4, 5, 6, 9, 11};
    for (unsigned i = 0; i < sizeof(scans)/sizeof(scans[0]); i++) {
        SDL_KeyboardEvent key = {0};
        RpEvent out = {0};
        key.keysym.scancode = (SDL_Scancode)scans[i];
        client_key_event(&key, &out);
        assert(out.kind == expected[i]);
        if (out.kind == 6) assert(out.value == 1);
    }
    SDL_KeyboardEvent key = {0};
    RpEvent out = {0};
    key.keysym.sym = SDLK_DOWN;
    client_key_event(&key, &out);
    assert(out.kind == 2);
    out.kind = 0;
    key.keysym.sym = SDLK_a;
    client_key_event(&key, &out);
    assert(out.kind == 0);
    /* Independent ABI fixture: bytes produced by sdl2-compat, not a struct
     * initialized with the consumer's headers (which hid the SDK mismatch). */
    SDL_Event event = {0};
    unsigned char raw[32] = {0};
    Uint32 type = SDL_KEYDOWN, scan = 81, symbol = 1073741905;
    memcpy(raw, &type, 4);
    raw[12] = SDL_PRESSED;
    raw[13] = 1;
    memcpy(raw + 16, &scan, 4);
    memcpy(raw + 20, &symbol, 4);
    memcpy(&event, raw, sizeof(raw));
    out.kind = 0;
    client_key_event(&event.key, &out);
    assert(out.kind == 2);
    assert(event.key.repeat == 1);
    assert(event.key.keysym.sym == SDLK_DOWN);
    return 0;
}
