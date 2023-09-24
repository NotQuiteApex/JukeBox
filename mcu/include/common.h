
#ifndef JUKEBOX_COMMON_H
#define JUKEBOX_COMMON_H

// Common includes
#include <pico/stdlib.h>
#include "config.h"


// Common macros
#define BIT(n) (1<<n)

#define REFRESH_CHECK(X, Y) \
    do { \
        static uint64_t start_ms = 0; \
        if ( time_us_64() / 1000 - start_ms - Y < X) { \
            return; \
        } \
        start_ms += X; \
    } while (0);


// Config checking
// TODO

#endif // JUKEBOX_COMMON_H
