#pragma once
#include <stddef.h>
int platform_open(void);
int platform_tick(void);
int platform_feed(const unsigned char *, size_t, int);
void platform_close(void);
int m0_run(void);
