#pragma once

#include <map>
#include <optional>
#include <set>
#include <string>
#include <vector>

namespace fog {

enum class TokenType {
    TERMINATOR,
    ASSIGN,
    LBRACE,
    RBRACE,
    LPAREN,
    RPAREN,

    IDENTIFIER,
    LET,
    CONST,
    INT,
    FLOAT,
    STRING,
    TRUE,
    FALSE,

    ARROW,
    COLON,
    COMMA,
    RETURN,

    IF,
    ELSE,
    WHILE,

    PLUS,
    MINUS,
    STAR,
    SLASH,

    EQ,
    NEQ,
    LT,
    LTE,
    GT,
    GTE
};

struct Token {
    TokenType type;
    std::string value;
    size_t pos;

    Token(TokenType type, std::string value, size_t pos)
        : type{type}, value{value}, pos{pos} {};
};

class Lexer {
   public:
    Lexer(std::string source) : source{source} {};
    std::vector<Token> Tokenize();

   private:
    std::string source;
    size_t pos = 0;
    int paren_depth = 0;
    int brace_depth = 0;

    void Next(int n = 1) { pos += n; }
    char Peek() { return source[pos]; }

    bool IsComment() {
        return pos + 1 < source.size() && source[pos] == '/' &&
               source[pos + 1] == '/';
    }

    Token ParseWord();
    Token ParseNumber();
    std::optional<Token> ParseTwoCharSymbol();
    std::optional<Token> ParseOneCharSymbol();
};

const std::map<std::string, TokenType> KEYWORD_TOKENS = {
    {"let", TokenType::LET},       {"const", TokenType::CONST},
    {"return", TokenType::RETURN}, {"if", TokenType::IF},
    {"else", TokenType::ELSE},     {"while", TokenType::WHILE},
    {"true", TokenType::TRUE},     {"false", TokenType::FALSE}};

const std::map<std::string, TokenType> TWO_CHAR_TOKENS = {
    {":=", TokenType::ASSIGN}, {"->", TokenType::ARROW}, {"!=", TokenType::NEQ},
    {"<=", TokenType::LTE},    {">=", TokenType::GTE},
};

const std::map<char, TokenType> ONE_CHAR_TOKENS = {
    {':', TokenType::COLON},  {';', TokenType::TERMINATOR},
    {'(', TokenType::LPAREN}, {')', TokenType::RPAREN},
    {'{', TokenType::LBRACE}, {'}', TokenType::RBRACE},
    {',', TokenType::COMMA},  {'+', TokenType::PLUS},
    {'-', TokenType::MINUS},  {'*', TokenType::STAR},
    {'/', TokenType::SLASH},  {'=', TokenType::EQ},
    {'<', TokenType::LT},     {'>', TokenType::GT}};

const std::set<TokenType> CONTINUATION_TOKENS = {
    TokenType::ARROW, TokenType::ASSIGN, TokenType::COLON, TokenType::COMMA,
    TokenType::PLUS,  TokenType::MINUS,  TokenType::STAR,  TokenType::SLASH,
    TokenType::EQ,    TokenType::NEQ,    TokenType::LT,    TokenType::LTE,
    TokenType::GT,    TokenType::GTE};

const std::map<TokenType, std::string> TOKEN_TYPE_NAMES = {
    {TokenType::TERMINATOR, "TERMINATOR"},
    {TokenType::ASSIGN, "ASSIGN"},
    {TokenType::LBRACE, "LBRACE"},
    {TokenType::RBRACE, "RBRACE"},
    {TokenType::LPAREN, "LPAREN"},
    {TokenType::RPAREN, "RPAREN"},
    {TokenType::IDENTIFIER, "IDENTIFIER"},
    {TokenType::LET, "LET"},
    {TokenType::CONST, "CONST"},
    {TokenType::INT, "INT"},
    {TokenType::FLOAT, "FLOAT"},
    {TokenType::STRING, "STRING"},
    {TokenType::TRUE, "TRUE"},
    {TokenType::FALSE, "FALSE"},
    {TokenType::ARROW, "ARROW"},
    {TokenType::COLON, "COLON"},
    {TokenType::COMMA, "COMMA"},
    {TokenType::RETURN, "RETURN"},
    {TokenType::IF, "IF"},
    {TokenType::ELSE, "ELSE"},
    {TokenType::WHILE, "WHILE"},
    {TokenType::PLUS, "PLUS"},
    {TokenType::MINUS, "MINUS"},
    {TokenType::STAR, "STAR"},
    {TokenType::SLASH, "SLASH"},
    {TokenType::EQ, "EQ"},
    {TokenType::NEQ, "NEQ"},
    {TokenType::LT, "LT"},
    {TokenType::LTE, "LTE"},
    {TokenType::GT, "GT"},
    {TokenType::GTE, "GTE"}};

}  // namespace fog