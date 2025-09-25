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
    ASTParser(std::vector<Token> tokens) : tokens{std::move(tokens)} {}
    std::unique_ptr<NodeBlock> ParseMain();

    std::unique_ptr<ASTNode> ParseStatement();
    std::unique_ptr<NodeBlock> ParseBlock();
    std::unique_ptr<NodeDeclare> ParseDeclare();
    std::unique_ptr<NodeAssign> ParseAssign();
    std::unique_ptr<NodeType> ParseType();
    std::unique_ptr<NodeExpr> ParseExpr(int);
    std::unique_ptr<NodeExpr> ParseExprPrimary();

   private:
    const std::map<TokenType, int> PRECEDENCE = {
        {TokenType::PLUS, 1},  {TokenType::MINUS, 1}, {TokenType::STAR, 2},
        {TokenType::SLASH, 2}, {TokenType::LT, 3},    {TokenType::LTE, 3},
        {TokenType::GT, 3},    {TokenType::GTE, 3},   {TokenType::EQ, 4},
        {TokenType::NEQ, 4}};

    std::vector<Token> tokens;
    size_t pos = 0;

    void Next() { pos++; }

    std::optional<Token> Peek() {
        if (pos >= tokens.size()) {
            return {};
        }
        return tokens[pos];
    }

    Token PeekRequired() {
        if (pos >= tokens.size()) {
            throw std::runtime_error("Unexpected EOF");
        }
        return tokens[pos];
    }

    bool Match(TokenType type) { return type == Peek().value().type; }

    Token Expect(TokenType type, char *err_msg) {
        Token tkn = Peek().value();
        if (tkn.type != type) {
            throw std::runtime_error(err_msg);
        }
        return tkn;
    }
};

}  // namespace fog