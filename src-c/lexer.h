#pragma once

#ifndef FOG_LEXER_H
#define FOG_LEXER_H

#include <stddef.h>

typedef enum
{
    FOG_TOKEN_NULL,

    FOG_TOKEN_TERMINATOR,
    FOG_TOKEN_ASSIGN,
    FOG_TOKEN_LBRACE,
    FOG_TOKEN_RBRACE,

    FOG_TOKEN_LPAREN,
    FOG_TOKEN_RPAREN,

    FOG_TOKEN_IDENTIFIER,
    FOG_TOKEN_LET,
    FOG_TOKEN_CONST,

    FOG_TOKEN_INT,
    FOG_TOKEN_FLOAT,
    FOG_TOKEN_STRING,

    FOG_TOKEN_TRUE,
    FOG_TOKEN_FALSE,

    FOG_TOKEN_ARROW,
    FOG_TOKEN_FATARROW,
    FOG_TOKEN_COLON,

    FOG_TOKEN_COMMA,
    FOG_TOKEN_RETURN,

    FOG_TOKEN_IF,
    FOG_TOKEN_ELSE,
    FOG_TOKEN_WHILE,

    FOG_TOKEN_PLUS,
    FOG_TOKEN_MINUS,
    FOG_TOKEN_STAR,
    FOG_TOKEN_SLASH,

    FOG_TOKEN_DIV,
    FOG_TOKEN_MOD,
    FOG_TOKEN_CARET,

    FOG_TOKEN_AND,
    FOG_TOKEN_OR,
    FOG_TOKEN_XOR,
    FOG_TOKEN_NOT,

    FOG_TOKEN_EQ,
    FOG_TOKEN_NEQ,

    FOG_TOKEN_LT,
    FOG_TOKEN_LTE,

    FOG_TOKEN_GT,
    FOG_TOKEN_GTE
}
FOG_TokenType;

typedef struct
{
    char *str;
    FOG_TokenType type;
}
FOG_TokenEntry;

typedef struct
{
    char *str;
    FOG_TokenType type;
    size_t pos;
}
FOG_Token;

FOG_Token fogGetNullToken();
FOG_Token fogGetTerminatorToken(size_t pos);

typedef struct
{
    FOG_Token *data;
    size_t size;
    size_t cap;
}
FOG_TokenList;

void fogTokenListInit(FOG_TokenList *ls);
void fogTokenListPush(FOG_TokenList *ls, FOG_Token token);

extern const FOG_TokenEntry FOG_KEYWORD_TOKENS[];
extern const FOG_TokenEntry FOG_TWO_CHAR_TOKENS[];
extern const FOG_TokenEntry FOG_ONE_CHAR_TOKENS[];
extern const FOG_TokenType FOG_CONTINUATION_TOKENS[];

typedef struct
{
    char *source;
    size_t sourceLen;
    size_t pos;
    int braceDepth;
    int parenDepth;
}
FOG_Lexer;

void fogLexerInit(FOG_Lexer *lexer, char *source, size_t sourceLen);

void fogLexerNext(FOG_Lexer *lexer);
char fogLexerPeek(FOG_Lexer *lexer);

FOG_Token fogLexerParseWord(FOG_Lexer *lexer);
FOG_Token fogLexerParseNumber(FOG_Lexer *lexer);

FOG_Token fogLexerParseTwoCharToken(FOG_Lexer *lexer);
FOG_Token fogLexerParseOneCharToken(FOG_Lexer *lexer);

int fogLexerIsComment(FOG_Lexer *lexer);

FOG_TokenList fogLexerTokenize(char *source, size_t sourceLen);

#endif