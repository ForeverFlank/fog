#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "types.h"
#include "lexer.h"

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

void fogLexerInit(FOG_Lexer *lexer, char *str, size_t strLen)
{
    lexer->str = str;
    lexer->strLen = strLen;
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
    return lexer->str[lexer->pos];
}

FOG_Token fogLexerParseWord(FOG_Lexer *lexer)
{
    size_t begin = lexer->pos;
    FOG_String word;
    fogStringInit(&word);

    char c = peek();
    while (isalnum(c) || c == '_')
    {
        fogStringPush(&word, c);
        next();
        c = peek();
    }

    FOG_Token token;
    token.str = word.buf;
    token.strLen = word.size;
    token.pos = begin;

    size_t arrSize = sizeof(FOG_KEYWORD_TOKENS) / sizeof(FOG_TokenEntry);
    for (int i = 0; i < arrSize; i++)
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

    char decimal = 0;
    // bool float64 = false;

    char c = peek();
    while (isdigit(c) || c == '.')
    {
        fogStringPush(&num, c);
        next();
        c = peek();

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
            throw std::runtime_error(
                "(" + std::to_string(pos) +
                ") Invalid number format: multiple decimal points");
        }
    }

    // if (Peek() == 'd') {
    //     float64 = true;
    //     Get();
    // }

    if (decimal)
    {
        return Token(TokenType::FLOAT, num, begin);
    }

    return Token(TokenType::INT, num, begin);
}

FOG_Token fogLexerParseOneCharSymbol(FOG_Lexer *lexer)
{
    return FOG_Token();
}

FOG_Token fogLexerParseTwoCharSymbol(FOG_Lexer *lexer)
{
    return FOG_Token();
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
        c = peek();

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
            next();
            continue;
        }
        if (is_comment())
        {
            while (lexer.pos < sourceLen && peek() != '\n') next();
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

        token = fogLexerParseTwoCharSymbol(&lexer);
        if (token.type != FOG_TOKEN_NULL)
        {
            fogTokenListPush(&tokens, token);
            continue;
        }

        token = fogLexerParseOneCharSymbol(&lexer);
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

        if (c == '\n' && lexer.parenDepth == 0)
        {
            FOG_TokenType lastTokenType = tokens.data[tokens.size - 1].type;
            size_t arrSize = sizeof(FOG_CONTINUATION_TOKENS) / sizeof(FOG_TokenType);

            char found = 0;
            for (int i = 0; i < arrSize && !found; i++)
            {
                if (FOG_CONTINUATION_TOKENS[i] == lastTokenType)
                    found = 1;
            }

            if (found)
            {
                FOG_Token token;
                token.type = FOG_TOKEN_TERMINATOR;
                token.str = NULL;
                token.strLen = 0;
                token.pos = lexer.pos;
                fogTokenListPush(&tokens, token);
                next();
                continue;
            }
        }

        next();
    }

    if (tokens.data[tokens.size - 1].type != FOG_TOKEN_TERMINATOR)
    {
        FOG_Token token;
        token.type = FOG_TOKEN_TERMINATOR;
        token.str = NULL;
        token.strLen = 0;
        token.pos = lexer.pos;
        fogTokenListPush(&tokens, token);
    }

    return tokens;
}