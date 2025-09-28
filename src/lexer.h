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
    FATARROW,
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
        : type{ type }, value{ value }, pos{ pos } { };
};

class Lexer {
public:
    Lexer(std::string source) : source{ source } { };
    std::vector<Token> tokenize();

private:
    std::string source;
    size_t pos = 0;
    int brace_depth = 0;
    int paren_depth = 0;

    void next() { pos++; }
    char peek() { return source[pos]; }

    bool is_comment() {
        return
            pos + 1 < source.size() &&
            source[pos] == '/' &&
            source[pos + 1] == '/';
    }

    Token parse_word();
    Token parse_number();
    std::optional<Token> parse_two_char_symbol();
    std::optional<Token> parse_one_char_symbol();
};

const std::map<std::string, TokenType> KEYWORD_TOKENS = {
    { "let",    TokenType::LET },       { "const",  TokenType::CONST },
    { "return", TokenType::RETURN },    { "if",     TokenType::IF },
    { "else",   TokenType::ELSE },      { "while",  TokenType::WHILE },
    { "do",     TokenType::LBRACE },    { "end",    TokenType::RBRACE },
    { "true",   TokenType::TRUE },      { "false",  TokenType::FALSE }
};

const std::map<std::string, TokenType> TWO_CHAR_TOKENS = {
    { ":=", TokenType::ASSIGN },    { "->", TokenType::ARROW },
    { "=>", TokenType::FATARROW },  { "!=", TokenType::NEQ },
    { "<=", TokenType::LTE },       { ">=", TokenType::GTE }
};

const std::map<char, TokenType> ONE_CHAR_TOKENS = {
    { ':', TokenType::COLON },      { ';', TokenType::TERMINATOR },
    { '(', TokenType::LPAREN },     { ')', TokenType::RPAREN },
    { '{', TokenType::LBRACE },     { '}', TokenType::RBRACE },
    { ',', TokenType::COMMA },      { '+', TokenType::PLUS },
    { '-', TokenType::MINUS },      { '*', TokenType::STAR },
    { '/', TokenType::SLASH },      { '=', TokenType::EQ },
    { '<', TokenType::LT },         { '>', TokenType::GT }
};

const std::set<TokenType> CONTINUATION_TOKENS = {
    TokenType::ARROW,   TokenType::ASSIGN,  TokenType::LBRACE,  TokenType::RBRACE,
    TokenType::COLON,   TokenType::COMMA,   TokenType::PLUS,    TokenType::MINUS,
    TokenType::STAR,    TokenType::SLASH,   TokenType::EQ,      TokenType::NEQ,
    TokenType::LT,      TokenType::LTE,     TokenType::GT,      TokenType::GTE
};

} // namespace fog