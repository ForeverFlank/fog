#include "lexer.h"

#include <map>
#include <optional>
#include <set>
#include <stdexcept>
#include <iostream>

namespace fog {

Token Lexer::ParseWord() {
    size_t begin = pos;
    std::string word;

    char c = Peek();
    while (isalnum(c) || c == '_') {
        word += c;
        Next();
        c = Peek();
    }

    auto it = KEYWORD_TOKENS.find(word);
    if (it != KEYWORD_TOKENS.end()) {
        return Token(it->second, word, begin);
    }

    return Token(TokenType::IDENTIFIER, word, begin);
}

Token Lexer::ParseNumber() {
    size_t begin = pos;
    std::string word;

    bool decimal = false;
    // bool float64 = false;

    char c = Peek();
    while (isdigit(c) || c == '.') {
        word += c;
        Next();
        c = Peek();
        
        if (c == '.') {
            if (!decimal)
                decimal = true;
            else
                throw std::runtime_error(
                    "(" + std::to_string(pos) +
                    ") Invalid number format: multiple decimal points");
        }
    }

    // if (Peek() == 'd') {
    //     float64 = true;
    //     Get();
    // }

    if (!decimal) {
        return Token(TokenType::INT, word, begin);
    }

    return Token(TokenType::FLOAT, word, begin);
}

std::optional<Token> Lexer::ParseTwoCharSymbol() {
    if (pos + 1 >= source.size()) return {};

    size_t begin = pos;
    std::string sym{source[pos], source[pos + 1]};

    auto it = TWO_CHAR_TOKENS.find(sym);
    if (it != TWO_CHAR_TOKENS.end()) {
        Next(2);
        return Token(it->second, sym, begin);
    }

    return {};
}

std::optional<Token> Lexer::ParseOneCharSymbol() {
    size_t begin = pos;
    char c = Peek();

    auto it = ONE_CHAR_TOKENS.find(c);
    if (it != ONE_CHAR_TOKENS.end()) {
        Next();
        return Token(it->second, std::string(1, c), begin);
    }

    return {};
}

std::vector<Token> Lexer::Tokenize() {
    size_t len = source.size();
    std::vector<Token> tokens;

    char c;
    while (pos < len) {
        c = Peek();

        if (paren_depth < 0)
            throw std::runtime_error("Parentheses depth cannot be negative");
        if (brace_depth < 0)
            throw std::runtime_error("Braces depth cannot be negative");

        if (c == ' ') {
            Next();
            continue;
        }
        if (IsComment()) {
            while (pos < len && Peek() != '\n') Next();
            continue;
        }
        if (isalpha(c) || c == '_') {
            tokens.push_back(ParseWord());
            continue;
        }
        if (isdigit(c)) {
            tokens.push_back(ParseNumber());
            continue;
        }

        std::optional<Token> res;
        
        res = ParseTwoCharSymbol();
        if (res.has_value()) {
            tokens.push_back(res.value());
        }

        res = ParseOneCharSymbol();
        if (res.has_value()) {
            tokens.push_back(res.value());

            switch (res.value().type) {
                case TokenType::LPAREN:
                    paren_depth++;
                    break;
                case TokenType::RPAREN:
                    paren_depth--;
                    break;
                case TokenType::LBRACE:
                    brace_depth++;
                    break;
                case TokenType::RBRACE:
                    brace_depth--;
                    break;
                default:
                    break;
            }
            continue;
        }

        if (c == '\n' && !CONTINUATION_TOKENS.contains(tokens.back().type)) {
            tokens.push_back(Token(TokenType::TERMINATOR, "", pos));
            Next();
            continue;
        }

        Next();
    }

    return tokens;
}

}  // namespace fog