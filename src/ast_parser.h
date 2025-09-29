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
    ASTParser(std::vector<Token> tokens) : tokens{ std::move(tokens) } { }
    std::unique_ptr<NodeBlock> parse_main();

private:
    const std::map<TokenType, int> OP_PRECEDENCE = {
        { TokenType::PLUS, 1 }, { TokenType::MINUS, 1 },
        { TokenType::STAR, 2 }, { TokenType::SLASH, 2 },
        { TokenType::LT,   3 }, { TokenType::LTE,   3 },
        { TokenType::GT,   3 }, { TokenType::GTE,   3 },
        { TokenType::EQ,   4 }, { TokenType::NEQ,   4 }
    };

    std::unique_ptr<ASTNode>     parse_statement();
    std::unique_ptr<NodeBlock>   parse_block();
    std::unique_ptr<NodeDeclare> parse_declare();
    std::unique_ptr<NodeAssign>  parse_assign();

    std::unique_ptr<NodeExpr> parse_expr(int min_prec = 0);
    std::unique_ptr<NodeExpr> parse_expr_primary();
    std::unique_ptr<NodeType> parse_type();
    std::unique_ptr<NodeType> parse_product_type();
    std::unique_ptr<NodeType> parse_sum_type();
    std::unique_ptr<NodeType> parse_type_primary();

    std::vector<Token> tokens;
    size_t pos = 0;

    void next() { pos++; }
    bool match(TokenType type) { return type == peek().type; }

    Token peek() {
        if (pos >= tokens.size()) {
            throw std::runtime_error("Unexpected EOF");
        }
        return tokens[pos];
    }

    Token expect(TokenType type, std::string err_msg) {
        Token tkn = peek();
        if (tkn.type != type) {
            throw std::runtime_error(err_msg.c_str());
        }
        return tkn;
    }
};

}  // namespace fog