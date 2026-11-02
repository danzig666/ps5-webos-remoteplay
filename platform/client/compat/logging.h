#pragma once
#include <stdio.h>
#define commons_log_info(tag, fmt, ...)                                        \
  fprintf(stderr, "HID %s " fmt "\n", tag, ##__VA_ARGS__)
#define commons_log_warn(tag, fmt, ...)                                        \
  fprintf(stderr, "HID %s " fmt "\n", tag, ##__VA_ARGS__)
