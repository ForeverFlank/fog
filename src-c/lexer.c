#include "lexer.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stddef.h>
#include <ctype.h>

#include "types.h"

FOG_Token fogGetNullToken()
{
    FOG_Token token;
    token.type = FOG_TOKEN_NULL;
    token.str = NULL;
    token.pos = 0;
    return token;
}

FOG_Token fogGetTerminatorToken(size_t pos)
{
    FOG_Token token;
    token.type = FOG_TOKEN_TERMINATOR;
    token.str = NULL;
    token.pos = pos;
    return token;
}

void fogTokenListInit(FOG_TokenList *ls)
{
    ls->data = NULL;
    ls->size = 0;
    ls->cap = 0;
}

void fogTokenListPush(FOG_TokenList *ls, FOG_Token token)
{
    if (ls->cap == 0)
    {
        ls->data = (FOG_Token *)malloc(sizeof(FOG_Token));
        ls->data[0] = token;
        ls->size = 1;
        ls->cap = 1;
        return;
    }

    if (ls->size + 1 > ls->cap)
    {
        ls->cap *= 2;
        FOG_Token *newData = (FOG_Token *)malloc(ls->cap * sizeof(FOG_Token));
        for (size_t i = 0; i < ls->size; i++)
        {
            newData[i] = ls->data[i];
        }
        free(ls->data);
        ls->data = newData;
    }
    ls->data[ls->size] = token;
    ls->size++;
}

const FOG_TokenEntry FOG_KEYWORD_TOKENS[] = {
    {"let",     FOG_TOKEN_LET},       {"const",   FOG_TOKEN_CONST},
    {"return",  FOG_TOKEN_RETURN},    {"if",      FOG_TOKEN_IF},
    {"else",    FOG_TOKEN_ELSE},      {"while",   FOG_TOKEN_WHILE},
    {"do",      FOG_TOKEN_LBRACE},    {"end",     FOG_TOKEN_RBRACE},
    {"true",    FOG_TOKEN_TRUE},      {"false",   FOG_TOKEN_FALSE},

    {"div",     FOG_TOKEN_DIV},       {"mod",    FOG_TOKEN_MOD},
    {"and",     FOG_TOKEN_AND},       {"or",     FOG_TOKEN_OR},
    {"xor",     FOG_TOKEN_XOR},       {"not",    FOG_TOKEN_NOT},
};

const FOG_TokenEntry FOG_TWO_CHAR_TOKENS[] = {
    {"<-", FOG_TOKEN_ASSIGN},     {"->", FOG_TOKEN_ARROW},
    {"=>", FOG_TOKEN_FATARROW},   {"!=", FOG_TOKEN_NEQ},
    {"<=", FOG_TOKEN_LTE},        {">=", FOG_TOKEN_GTE},
};

const FOG_TokenEntry FOG_ONE_CHAR_TOKENS[] = {
    {":", FOG_TOKEN_COLON},       {";", FOG_TOKEN_TERMINATOR},
    {"(", FOG_TOKEN_LPAREN},      {")", FOG_TOKEN_RPAREN},
    {"{", FOG_TOKEN_LBRACE},      {"}", FOG_TOKEN_RBRACE},
    {",", FOG_TOKEN_COMMA},       {"+", FOG_TOKEN_PLUS},
    {"-", FOG_TOKEN_MINUS},       {"*", FOG_TOKEN_STAR},
    {"/", FOG_TOKEN_SLASH},       {"^", FOG_TOKEN_CARET},
    {"<", FOG_TOKEN_LT},          {">", FOG_TOKEN_GT},
    {"=", FOG_TOKEN_EQ},
};

const FOG_TokenType FOG_CONTINUATION_TOKENS[] = {
    FOG_TOKEN_ARROW,  FOG_TOKEN_ASSIGN, FOG_TOKEN_LBRACE, FOG_TOKEN_RBRACE,
    FOG_TOKEN_COLON,  FOG_TOKEN_COMMA,  FOG_TOKEN_PLUS,   FOG_TOKEN_MINUS,
    FOG_TOKEN_STAR,   FOG_TOKEN_SLASH,  FOG_TOKEN_EQ,     FOG_TOKEN_NEQ,
    FOG_TOKEN_LT,     FOG_TOKEN_LTE,    FOG_TOKEN_GT,     FOG_TOKEN_GTE
};

void fogLexerInit(FOG_Lexer *lexer, char *source, size_t sourceLen)
{
    lexer->source = source;
    lexer->sourceLen = sourceLen;
    lexer->pos = 0;
    lexer->braceDepth = 0;
    lexer->parenDepth = 0;
}

void fogLexerNext(FOG_Lexer *lexer)
{
    lexer->pos++;
}

char fogLexerPeek(FOG_Lexer *lexer)
{
    return lexer->source[lexer->pos];
}

FOG_Token fogLexerParseWord(FOG_Lexer *lexer)
{
    size_t begin = lexer->pos;
    FOG_String word;
    fogStringInit(&word);

    char c = fogLexerPeek(lexer);
    while (isalnum(c) || c == '_')
    {
        fogStringPush(&word, c);
        fogLexerNext(lexer);
        c = fogLexerPeek(lexer);
    }
    fogStringPush(&word, '\0');

    FOG_Token token;
    token.str = word.buf;
    token.pos = begin;

    size_t arrSize = sizeof(FOG_KEYWORD_TOKENS) / sizeof(FOG_TokenEntry);
    for (size_t i = 0; i < arrSize; i++)
    {
        FOG_TokenEntry entry = FOG_KEYWORD_TOKENS[i];
        size_t entryStrLen = sizeof(entry.str);

        if (word.size == entryStrLen &&
            memcmp(word.buf, entry.str, word.size) == 0)
        {
            token.type = entry.type;
            return token;
        }
    }

    token.type = FOG_TOKEN_IDENTIFIER;
    return token;
}

FOG_Token fogLexerParseNumber(FOG_Lexer *lexer)
{
    size_t begin = lexer->pos;
    FOG_String num;
    fogStringInit(&num);

    char decimal = 0;
    // bool float64 = false;

    char c = fogLexerPeek(lexer);
    while (isdigit(c) || c == '.')
    {
        fogStringPush(&num, c);
        fogLexerNext(lexer);
        c = fogLexerPeek(lexer);

        if (c != '.')
        {
            continue;
        }

        if (!decimal)
        {
            decimal = 1;
        }
        else
        {
            fprintf(stderr, "(%ld) Invalid number format: multiple decimal points", lexer->pos);
        }
    }

    // if (Peek() == 'd') {
    //     float64 = true;
    //     Get();
    // }

    fogStringPush(&num, '\0');

    FOG_Token token;
    token.pos = begin;
    token.str = num.buf;
    token.type = decimal ? FOG_TOKEN_FLOAT : FOG_TOKEN_INT;

    return token;
}

FOG_Token fogLexerParseTwoCharToken(FOG_Lexer *lexer)
{
    if (lexer->pos + 1 >= lexer->sourceLen)
    {
        return fogGetNullToken();
    }

    size_t begin = lexer->pos;
    char *sym = (char *)malloc(3);
    sym[0] = lexer->source[lexer->pos];
    sym[1] = lexer->source[lexer->pos + 1];
    sym[2] = '\0';

    size_t arrSize = sizeof(FOG_TWO_CHAR_TOKENS) / sizeof(FOG_TokenEntry);
    for (size_t i = 0; i < arrSize; i++)
    {
        FOG_TokenEntry entry = FOG_TWO_CHAR_TOKENS[i];
        if (strcmp(entry.str, sym) == 0)
        {
            fogLexerNext(lexer);
            fogLexerNext(lexer);

            FOG_Token token;
            token.type = entry.type;
            token.str = sym;
            token.pos = begin;
            return token;
        }
    }

    return fogGetNullToken();
}

FOG_Token fogLexerParseOneCharToken(FOG_Lexer *lexer)
{
    size_t begin = lexer->pos;
    char *c = (char *)malloc(2);
    c[0] = fogLexerPeek(lexer);
    c[1] = '\0';

    size_t arrSize = sizeof(FOG_ONE_CHAR_TOKENS) / sizeof(FOG_TokenEntry);
    for (size_t i = 0; i < arrSize; i++)
    {
        FOG_TokenEntry entry = FOG_ONE_CHAR_TOKENS[i];
        if (strcmp(entry.str, c) == 0)
        {
            fogLexerNext(lexer);

            FOG_Token token;
            token.type = entry.type;
            token.str = c;
            token.pos = begin;
            return token;
        }
    }

    return fogGetNullToken();
}

int fogLexerIsComment(FOG_Lexer *lexer)
{
    return
        lexer->pos + 1 < lexer->sourceLen &&
        lexer->source[lexer->pos] == '/' &&
        lexer->source[lexer->pos + 1] == '/';
}

FOG_TokenList fogLexerTokenize(char *source, size_t sourceLen)
{
    FOG_Lexer lexer;
    fogLexerInit(&lexer, source, sourceLen);

    FOG_TokenList tokens;
    fogTokenListInit(&tokens);

    char c;
    while (lexer.pos < sourceLen)
    {
        c = fogLexerPeek(&lexer);

        if (lexer.parenDepth < 0)
        {
            fprintf(stderr, "Parentheses depth cannot be negative");
            return tokens;
        }
        if (lexer.braceDepth < 0)
        {
            fprintf(stderr, "Braces depth cannot be negative");
            return tokens;
        }

        if (c == ' ')
        {
            fogLexerNext(&lexer);
            continue;
        }
        if (fogLexerIsComment(&lexer))
        {
            while (lexer.pos < sourceLen && fogLexerPeek(&lexer) != '\n')
            {
                fogLexerNext(&lexer);
            }
            continue;
        }
        if (isalpha(c) || c == '_')
        {
            fogTokenListPush(&tokens, fogLexerParseWord(&lexer));
            continue;
        }
        if (isdigit(c))
        {
            fogTokenListPush(&tokens, fogLexerParseNumber(&lexer));
            continue;
        }

        FOG_Token token;

        token = fogLexerParseTwoCharToken(&lexer);
        if (token.type != FOG_TOKEN_NULL)
        {
            fogTokenListPush(&tokens, token);
            continue;
        }

        token = fogLexerParseOneCharToken(&lexer);
        if (token.type != FOG_TOKEN_NULL)
        {
            fogTokenListPush(&tokens, token);

            switch (token.type)
            {
            case FOG_TOKEN_LBRACE:
                lexer.braceDepth++;
                break;
            case FOG_TOKEN_RBRACE:
                lexer.braceDepth--;
                break;
            case FOG_TOKEN_LPAREN:
                lexer.parenDepth++;
                break;
            case FOG_TOKEN_RPAREN:
                lexer.parenDepth--;
                break;
            default:
                break;
            }
            continue;
        }

        // check if the last token of the line is not a 
        // continuation token. if so, push a terminator token
        // to the list.
        if (c == '\n' && lexer.parenDepth == 0 && tokens.size > 0)
        {
            FOG_TokenType lastTokenType = tokens.data[tokens.size - 1].type;
            size_t arrSize = sizeof(FOG_CONTINUATION_TOKENS) / sizeof(FOG_TokenType);

            int found = 0;
            for (size_t i = 0; i < arrSize && !found; i++)
            {
                if (FOG_CONTINUATION_TOKENS[i] == lastTokenType)
                {
                    found = 1;
                }
            }

            if (!found)
            {
                FOG_Token token = fogGetTerminatorToken(lexer.pos);
                fogLexerNext(&lexer);
                fogTokenListPush(&tokens, token);
                continue;
            }
        }

        fogLexerNext(&lexer);
    }

    // push an extra terminator token at the end, if not already existed
    if (tokens.data[tokens.size - 1].type != FOG_TOKEN_TERMINATOR)
    {
        FOG_Token token = fogGetTerminatorToken(lexer.pos);
        fogTokenListPush(&tokens, token);
    }

    return tokens;
}
