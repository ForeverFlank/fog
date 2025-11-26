#include "types.h"

#include <stdio.h>
#include <stdlib.h>

void fogStringInit(FOG_String *str)
{ 
    str->buf = NULL;
    str->size = 0;
    str->cap = 0;
}

void fogStringPush(FOG_String *str, char c)
{
    if (str->cap == 0)
    {
        str->buf = (char *)malloc(sizeof(char));
        str->buf[0] = c;
        str->size = 1;
        str->cap = 1;
        return;
    }

    if (str->size + 1 > str->cap)
    {
        str->cap *= 2;
        char *newBuf = (char *)malloc(str->cap * sizeof(char));
        for (size_t i = 0; i < str->size; i++)
        {
            newBuf[i] = str->buf[i];
        }
        free(str->buf);
        str->buf = newBuf;
    }
    str->buf[str->size] = c;
    str->size++;
}