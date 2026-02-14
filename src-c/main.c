#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "types.h"
#include "lexer.h"

const FOG_TokenEntry TOKEN_TYPE_NAMES[] = {
    {"TERMINATOR",  FOG_TOKEN_TERMINATOR },
    {"ASSIGN",      FOG_TOKEN_ASSIGN     },
    {"LBRACE",      FOG_TOKEN_LBRACE     },
    {"RBRACE",      FOG_TOKEN_RBRACE     },
    {"LPAREN",      FOG_TOKEN_LPAREN     },
    {"RPAREN",      FOG_TOKEN_RPAREN     },
    {"IDENTIFIER",  FOG_TOKEN_IDENTIFIER },
    {"LET",         FOG_TOKEN_LET        },
    {"CONST",       FOG_TOKEN_CONST      },
    {"INT",         FOG_TOKEN_INT        },
    {"FLOAT",       FOG_TOKEN_FLOAT      },
    {"STRING",      FOG_TOKEN_STRING     },
    {"TRUE",        FOG_TOKEN_TRUE       },
    {"FALSE",       FOG_TOKEN_FALSE      },
    {"ARROW",       FOG_TOKEN_ARROW      },
    {"FATARROW",    FOG_TOKEN_FATARROW   },
    {"COLON",       FOG_TOKEN_COLON      },
    {"COMMA",       FOG_TOKEN_COMMA      },
    {"RETURN",      FOG_TOKEN_RETURN     },
    {"IF",          FOG_TOKEN_IF         },
    {"ELSE",        FOG_TOKEN_ELSE       },
    {"WHILE",       FOG_TOKEN_WHILE      },
    {"PLUS",        FOG_TOKEN_PLUS       },
    {"MINUS",       FOG_TOKEN_MINUS      },
    {"STAR",        FOG_TOKEN_STAR       },
    {"SLASH",       FOG_TOKEN_SLASH      },
    {"EQ",          FOG_TOKEN_EQ         },
    {"NEQ",         FOG_TOKEN_NEQ        },
    {"LT",          FOG_TOKEN_LT         },
    {"LTE",         FOG_TOKEN_LTE        },
    {"GT",          FOG_TOKEN_GT         },
    {"GTE",         FOG_TOKEN_GTE        }
};

void printTokens(FOG_TokenList *tokens)
{
    for (size_t i = 0; i < tokens->size; i++)
    {
        FOG_Token currToken = tokens->data[i];

        size_t arrSize = sizeof(TOKEN_TYPE_NAMES) / sizeof(FOG_TokenEntry);
        for (size_t j = 0; j < arrSize; j++)
        {
            FOG_TokenEntry tokenEntry = TOKEN_TYPE_NAMES[j];

            if (currToken.type != tokenEntry.type)
                continue;

            printf("%4ld %12s | %s\n", i, tokenEntry.str, currToken.str);
        }
    }
}

char *loadFile(char *path, size_t *len)
{
    FOG_String str;
    fogStringInit(&str);

    FILE *fp = fopen(path, "r");

    char c;
    while ((c = fgetc(fp)) != EOF)
    {
        fogStringPush(&str, c);
    }
    fogStringPush(&str, '\0');

    *len = str.size;
    return str.buf;
}

int main(int argc, char **argv)
{
    if (argc < 2)
    {
        fprintf(stderr, "usage: %s <file>\n", argv[0]);
        return 1;
    }

    char *path = argv[1];
    size_t sourceLen = 0;

    char *source = loadFile(path, &sourceLen);
    if (!source)
    {
        fprintf(stderr, "Failed to open file: %s\n", path);
        return 1;
    }

    FOG_TokenList tokens = fogLexerTokenize(source, sourceLen);

    printTokens(&tokens);

    // ASTParser parser = ast_parser_make(tokens.data, tokens.count);
    // NodeBlock *main_block = ast_parse_main(&parser);

    // print_ast(main_block);

    // Interpreter interp;
    // interpreter_init(&interp);
    // interpreter_eval(&interp, main_block);

    free(source);
    free(tokens.data);

    return 0;
}