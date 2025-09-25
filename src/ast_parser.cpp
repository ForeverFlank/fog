#include "ast_parser.h"

#include <iostream>
#include <memory>
#include <sstream>
#include <string>

#include "ast_nodes.h"

namespace fog {

std::unique_ptr<NodeBlock> ASTParser::ParseMain() {
    std::vector<std::unique_ptr<ASTNode>> statements;

    std::unique_ptr<ASTNode> stmt;
    while (Peek().has_value()) {
        stmt = std::move(ParseStatement());

        if (stmt != nullptr) {
            statements.push_back(std::move(stmt));
        }
    }

    return std::make_unique<NodeBlock>(std::move(statements));
}

std::unique_ptr<ASTNode> ASTParser::ParseStatement() {
    auto optional_tkn = Peek();

    if (!optional_tkn.has_value()) {
        return nullptr;
    }

    TokenType type = optional_tkn.value().type;

    // if (type == TokenType::LBRACE) {
    //     return ParseBlock();
    // }

    if (type == TokenType::LET || type == TokenType::CONST) {
        return ParseDeclare();
    }

    if (type == TokenType::IDENTIFIER && pos + 1 < tokens.size() &&
        tokens[pos + 1].type == TokenType::ASSIGN) {
        return ParseAssign();
    }

    return nullptr;
}

std::unique_ptr<NodeBlock> ASTParser::ParseBlock() {
    bool inside_brace = false;
    std::vector<std::unique_ptr<ASTNode>> statements;

    std::unique_ptr<ASTNode> stmt;
    while (Peek().has_value()) {
        stmt = ParseStatement();

        if (stmt != nullptr) {
            statements.push_back(std::move(stmt));
        }
    }

    return std::make_unique<NodeBlock>(std::move(statements));
}

std::unique_ptr<NodeDeclare> ASTParser::ParseDeclare() {
    bool is_let = Match(TokenType::LET);
    bool is_const = Match(TokenType::CONST);

    if (!is_let && !is_const) {
        throw std::runtime_error("Expected 'let' or 'const'");
    }
    Next();

    Token var_tkn = Peek().value();
    Next();

    Expect(TokenType::COLON, "Expected ':'");
    Next();

    auto type_node = std::move(ParseType());

    auto var_node = std::make_unique<NodeVariable>(var_tkn.value, std::move(type_node));

    if (Match(TokenType::TERMINATOR)) {
    }

    Expect(TokenType::ASSIGN, "Expected ':='");
    Next();

    auto value_node = std::move(ParseExpr(0));

    return std::make_unique<NodeDeclare>(is_const, std::move(var_node), std::move(value_node));
}

std::unique_ptr<NodeAssign> ASTParser::ParseAssign() {}

std::unique_ptr<NodeType> ASTParser::ParseType() { return nullptr; }

std::unique_ptr<NodeExpr> ASTParser::ParseExpr(int min_prec) {
    auto expr = ParseExprPrimary();

    while (Peek().has_value()) {
        Token op = Peek().value();
        auto it = PRECEDENCE.find(op.type);
        if (it == PRECEDENCE.end()) {
            break;
        }
        int prec = PRECEDENCE.at(op.type);
        if (prec < min_prec) {
            break;
        }
        Next();
        auto rhs = std::move(ParseExpr(prec + 1));
        expr = std::make_unique<NodeBinaryOp>(op.value, std::move(expr), std::move(rhs));
    }

    return expr;
}

std::unique_ptr<NodeExpr> ASTParser::ParseExprPrimary() {
    auto tkn = PeekRequired();
    Next();

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

}  // namespace fog