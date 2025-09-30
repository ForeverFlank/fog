#pragma once

#include <algorithm>
#include <optional>
#include <stdexcept>
#include <unordered_map>
#include <unordered_set>
#include <vector>

#include "ast_nodes.h"
#include "lexer.h"

namespace fog {

class ASTParser {
public:
    ASTParser(std::vector<Token> tokens) : tokens{std::move(tokens)} { }
    std::unique_ptr<NodeBlock> parse_main();

private:
    const std::unordered_set<TokenType> UNARY_OP = {
        TokenType::NOT, TokenType::MINUS
    };
    
    const std::unordered_map<TokenType, int> BINARY_OP_PRECEDENCE = {
        {TokenType::EQ,     8}, {TokenType::NEQ,   8},
        
        {TokenType::LT,     7}, {TokenType::LTE,   7},
        {TokenType::GT,     7}, {TokenType::GTE,   7},
        
        {TokenType::CARET,  6},
        
        {TokenType::STAR,   5}, {TokenType::SLASH, 5},
        {TokenType::DIV,    5}, {TokenType::MOD,   5},
        
        {TokenType::PLUS,   4}, {TokenType::MINUS, 4},
        
        {TokenType::AND,    3},
        {TokenType::XOR,    2},
        {TokenType::OR,     1},
    };

    const std::unordered_set<TokenType> RIGHT_ASSOC_OP = {
        TokenType::CARET,
    };

    std::unique_ptr<ASTNode>     parse_statement();
    std::unique_ptr<NodeBlock>   parse_block();
    std::unique_ptr<NodeDeclare> parse_declare();
    std::unique_ptr<NodeAssign>  parse_assign();

    std::unique_ptr<NodeExpr> parse_expr(int min_prec = 0);
    std::unique_ptr<NodeExpr> parse_expr_primary();
    std::unique_ptr<NodeExpr> parse_expr_unary(Token tkn);
    std::unique_ptr<NodeExpr> parse_expr_literal(Token tkn);

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