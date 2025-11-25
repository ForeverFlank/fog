#pragma once

#include <inttypes.h>

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
    size_t strLen;
    FOG_TokenType type;
    size_t pos;
}
FOG_Token;

typedef struct
{
    FOG_Token *data;
    size_t size;
    size_t cap;
}
FOG_TokenList;

void fogTokenListInit(FOG_TokenList *ls);
void fogTokenListPush(FOG_TokenList *ls, FOG_Token token);

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
    {":=", FOG_TOKEN_ASSIGN},     {"->", FOG_TOKEN_ARROW},
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

typedef struct
{
    char *str;
    size_t strLen;
    size_t pos;
    size_t braceDepth;
    size_t parenDepth;
}
FOG_Lexer;

void fogLexerInit(FOG_Lexer *lexer, char *str, size_t strLen);

void fogLexerNext(FOG_Lexer *lexer);
char fogLexerPeek(FOG_Lexer *lexer);

FOG_Token fogLexerParseWord(FOG_Lexer *lexer);
FOG_Token fogLexerParseNumber(FOG_Lexer *lexer);

FOG_Token fogLexerParseOneCharSymbol(FOG_Lexer *lexer);
FOG_Token fogLexerParseTwoCharSymbol(FOG_Lexer *lexer);

FOG_TokenList fogLexerTokenize(char *str, size_t strLen);