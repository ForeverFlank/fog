#pragma once

#include "ast_parser.h"

#include <typeinfo>

#include "ast_nodes.h"

namespace fog {

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
            statements.push_back(stmt);
        }
    }

    return std::make_unique<NodeBlock>(NodeBlock(statements));
}

std::unique_ptr<NodeDeclare> ASTParser::ParseDeclare() {

}

std::unique_ptr<NodeAssign> ASTParser::ParseAssign() {

}

std::unique_ptr<NodeType> ASTParser::ParseType() {

}

std::unique_ptr<NodeExpr> ASTParser::ParseExpr() {

}

}  // namespace fog