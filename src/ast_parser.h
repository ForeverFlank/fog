#pragma once

#include <algorithm>
#include <optional>
#include <stdexcept>
#include <vector>

#include "ast_nodes.h"
#include "lexer.h"

namespace fog {

class ASTParser {
   public:
    ASTParser(std::vector<Token> tokens) : tokens{tokens} {}

    std::unique_ptr<ASTNode> ParseStatement() {}
    std::unique_ptr<NodeBlock> ParseBlock() {}
    std::unique_ptr<NodeDeclare> ParseDeclare() {}
    std::unique_ptr<NodeAssign> ParseAssign() {}
    std::unique_ptr<NodeType> ParseType() {}
    std::unique_ptr<NodeExpr> ParseExpr() {}

   private:
    std::vector<Token> tokens;
    size_t pos = 0;

    void Next() { pos++; }

    std::optional<Token> Peek() {
        if (pos >= tokens.size()) {
            throw std::runtime_error("Unexpected EOF");
        }
        return tokens[pos];
    }

    bool Match(std::vector<TokenType> types) {
        TokenType type = Peek().value().type;
        auto it = std::find(types.begin(), types.end(), type);
        return it != types.end();
    }

    Token Expect(TokenType type, char *err_msg) {
        Token tkn = Peek().value();
        if (tkn.type != type) {
            throw std::runtime_error(err_msg);
        }
        return tkn;
    }
};

}  // namespace fog