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
    char *pref = SDL_GetPrefPath("homebrew", "ps5remote");
    if (!pref) {
        fprintf(stderr, "Cannot get private data path\n");
        return 1;
    }
    if (!getenv("PS5REMOTE_DATA_DIR"))
        setenv("PS5REMOTE_DATA_DIR", pref, 1);
    SDL_free(pref);
    fprintf(stderr, "PS5 Remote 0.2.0: hardware status UNVERIFIED uid=%ld\n",
            (long)getuid());
    return rp_run();
}
