#include "../platform/client/navigation.h"
#include <assert.h>
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
    return 0;
}
