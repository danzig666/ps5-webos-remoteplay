#include <assert.h>
extern int probe_grab(const char *);
int main(void) {
    assert(probe_grab(0)==-1);
    assert(probe_grab("/tmp/m0-do-not-grab")==-1);
    assert(probe_grab("/dev/input/event-m0-nonexistent")==-1);
    return 0;
}
