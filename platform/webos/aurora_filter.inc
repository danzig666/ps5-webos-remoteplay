/* Extracted unchanged from Aurora-TV input_gamepad.c; GPL-3.0, see THIRD_PARTY.md. */
static bool str_contains_ci(const char *haystack, const char *needle) {
    if (haystack == NULL || needle == NULL || *needle == '\0') {
        return false;
    }
    size_t needle_len = strlen(needle);
    for (const char *cur = haystack; *cur != '\0'; cur++) {
        size_t i = 0;
        while (i < needle_len && cur[i] != '\0' &&
               tolower((unsigned char) cur[i]) == tolower((unsigned char) needle[i])) {
            i++;
        }
        if (i == needle_len) {
            return true;
        }
    }
    return false;
}

static bool webos_name_is_non_gamepad(const char *name) {
    return str_contains_ci(name, "remote") ||
           str_contains_ci(name, "keyboard") ||
           str_contains_ci(name, "mouse") ||
           str_contains_ci(name, "pointer") ||
           str_contains_ci(name, "touch") ||
           str_contains_ci(name, "air mouse") ||
           str_contains_ci(name, "webos") ||
           str_contains_ci(name, "lge") ||
           str_contains_ci(name, "lg electronics");
}

