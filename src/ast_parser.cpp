#include "ast_parser.h"

#include <iostream>
#include <memory>
#include <sstream>
#include <string>

#include "ast_nodes.h"

namespace fog {

std::unique_ptr<NodeBlock> ASTParser::parse_main() {
    std::vector<std::unique_ptr<ASTNode>> statements;

    std::unique_ptr<ASTNode> stmt;
    while (peek().has_value()) {
        // std::cout << pos << " ";

        if (peek().value().type == TokenType::TERMINATOR) {
            next();
            continue;
        }

        stmt = std::move(parse_statement());
        
        if (stmt != nullptr) {
            statements.push_back(std::move(stmt));
        }
    }

    return std::make_unique<NodeBlock>(std::move(statements));
}

std::unique_ptr<ASTNode> ASTParser::parse_statement() {
    auto optional_tkn = peek();

    if (!optional_tkn.has_value()) {
        return nullptr;
    }

    TokenType type = optional_tkn.value().type;

    // if (type == TokenType::LBRACE) {
    //     return ParseBlock();
    // }

    if (type == TokenType::LET || type == TokenType::CONST) {
        return parse_declare();
    }

    if (type == TokenType::IDENTIFIER && pos + 1 < tokens.size() &&
        tokens[pos + 1].type == TokenType::ASSIGN) {
        return parse_assign();
    }

    // next();
    return nullptr;
}

std::unique_ptr<NodeBlock> ASTParser::parse_block() {
    bool inside_brace = false;
    std::vector<std::unique_ptr<ASTNode>> statements;

    std::unique_ptr<ASTNode> stmt;
    while (peek().has_value()) {
        stmt = parse_statement();

        if (stmt != nullptr) {
            statements.push_back(std::move(stmt));
        }
    }

    return std::make_unique<NodeBlock>(std::move(statements));
}

std::unique_ptr<NodeDeclare> ASTParser::parse_declare() {
    bool is_let = match(TokenType::LET);
    bool is_const = match(TokenType::CONST);

    if (!is_let && !is_const) {
        throw std::runtime_error("Expected 'let' or 'const'");
    }
    next();

    Token var_tkn = peek().value();
    next();

    expect(TokenType::COLON, "Expected ':'");
    next();

    auto type_node = parse_type();

    auto var_node = std::make_unique<NodeVariable>(
        var_tkn.value,
        std::move(type_node)
    );

    std::unique_ptr<NodeExpr> value_node = nullptr;

    if (!match(TokenType::TERMINATOR)) {
        expect(TokenType::ASSIGN, "Expected ':='");
        next();

        value_node = parse_expr(0);
    }

    return std::make_unique<NodeDeclare>(
        is_const,
        std::move(var_node),
        std::move(value_node)
    );
}

std::unique_ptr<NodeAssign> ASTParser::parse_assign() {
    return nullptr;
}

std::unique_ptr<NodeType> ASTParser::parse_type() {
    next();
    return nullptr;
}

std::unique_ptr<NodeExpr> ASTParser::parse_expr(int min_prec) {
    auto expr = parse_expr_primary();

    while (peek().has_value()) {
        Token op = peek().value();
        auto it = PRECEDENCE.find(op.type);
        if (it == PRECEDENCE.end()) {
            break;
        }

        int prec = PRECEDENCE.at(op.type);
        if (prec < min_prec) {
            break;
        }
        next();

        auto rhs = parse_expr(prec + 1);
        expr = std::make_unique<NodeBinaryOp>(
            op.value,
            std::move(expr),
            std::move(rhs)
        );
    }

    return expr;
}

std::unique_ptr<NodeExpr> ASTParser::parse_expr_primary() {
    auto tkn = peek_required();
    next();

    std::string str = tkn.value;
    std::stringstream ss(str);

    if (tkn.type == TokenType::INT) {
        int64_t val;
        ss >> val;
        return std::make_unique<NodeInt64Literal>(val);
    }

    if (tkn.type == TokenType::FLOAT) {
        float val;
        ss >> val;
        return std::make_unique<NodeFloatLiteral>(val);
    }

    throw std::runtime_error("Unexpected token: " + str);
}

} // namespace fog