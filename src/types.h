#pragma once

#include <inttypes.h>

typedef struct
{
    char *buf;
    size_t size;
    size_t cap;
}
FOG_String;

void fogStringInit(FOG_String *str);

void fogStringPush(FOG_String *str, char c);