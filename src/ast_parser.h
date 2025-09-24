#pragma once

#include <algorithm>
#include <stdexcept>
#include <vector>

#include "ast_nodes.h"
#include "lexer.h"

namespace fog {

class ASTParser {
   public:
    ASTParser(std::vector<Token> tokens) : tokens{tokens} {}

   private:
    std::vector<Token> tokens;
    size_t pos = 0;

    void Next() { pos++; }
    Token Peek() { return tokens[pos]; }

    bool Match(std::vector<TokenType> types) {
        auto it = std::find(types.begin(), types.end(), Peek().type);
        return it != types.end();
    }

    Token Expect(TokenType type, char *err_msg) {
        Token tkn = Peek();
        if (tkn.type != type) {
            throw std::runtime_error(err_msg);
        }
        return tkn;
    }
};

}  // namespace fog