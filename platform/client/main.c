// SPDX-License-Identifier: AGPL-3.0-only
#define _GNU_SOURCE
#include "client.h"
#include <limits.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <unistd.h>
int main(void) {
    char path[PATH_MAX];
    ssize_t n = readlink("/proc/self/exe", path, sizeof(path) - 1);
    if (n < 0 || n >= (ssize_t)sizeof(path) - 1)
        return 1;
    path[n] = 0;
    char *s = strrchr(path, '/');
    if (!s)
        return 1;
    *s = 0;
    if (chdir(path))
        return 1;
    umask(0077);
    setenv("APPID", "org.homebrew.ps5remote", 1);
    /* Bounded per-launch log rotation; never log configuration fields or input
     * text. */
    rename("/tmp/ps5remote.log", "/tmp/ps5remote.log.previous");
    if (!freopen("/tmp/ps5remote.log", "w", stderr))
        return 1;
    setvbuf(stderr, NULL, _IOLBF, 0);
    fprintf(stderr, "Startup dev.4: bootstrap reached uid=%ld\n", (long)getuid());
    fprintf(stderr, "Startup resolving private data directory\n");
    char *pref = SDL_GetPrefPath("homebrew", "ps5remote");
    if (!pref) {
        fprintf(stderr, "Cannot get private data path\n");
        return 1;
    }
    if (!getenv("PS5REMOTE_DATA_DIR")) {
        /* SDL webOS returns the package directory, which the installer owns.
         * Create our own private child; never chmod the installation root. */
        int length = snprintf(path, sizeof(path), "%s%sprivate", pref,
                              pref[0] && pref[strlen(pref) - 1] == '/' ? "" : "/");
        if (length < 0 || (size_t)length >= sizeof(path) ||
            setenv("PS5REMOTE_DATA_DIR", path, 1) != 0) {
            SDL_free(pref);
            fprintf(stderr, "Cannot select private data directory\n");
            return 1;
        }
    }
    SDL_free(pref);
    fprintf(stderr, "Startup private data directory resolved; entering Rust\n");
    fprintf(stderr, "PS5 Remote 0.2.0: hardware status UNVERIFIED uid=%ld\n",
            (long)getuid());
    return rp_run();
}
