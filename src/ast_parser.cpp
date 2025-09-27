#include "ast_parser.h"

#include <iostream>
#include <memory>
#include <sstream>
#include <string>
#include <variant>

#include "ast_nodes.h"

namespace fog {

std::unique_ptr<NodeBlock> ASTParser::parse_main() {
    std::vector<std::unique_ptr<ASTNode>> statements;

    std::unique_ptr<ASTNode> stmt;
    while (pos < tokens.size()) {
        if (peek().type == TokenType::TERMINATOR) {
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
    if (pos >= tokens.size()) {
        return nullptr;
    }

    TokenType type = peek().type;

    if (type == TokenType::TERMINATOR) {
        next();
        return nullptr;
    }

    if (type == TokenType::LBRACE) {
        return parse_block();
    }

    if (type == TokenType::LET || type == TokenType::CONST) {
        return parse_declare();
    }

    if (type == TokenType::IDENTIFIER && pos + 1 < tokens.size() &&
        tokens[pos + 1].type == TokenType::ASSIGN) {
        return parse_assign();
    }

    if (type == TokenType::RETURN) {
        next();
        auto expr = parse_expr();
        return std::make_unique<NodeReturn>(std::move(expr));
    }

    throw std::runtime_error("Unexpected token: " + peek().value);
}

std::unique_ptr<NodeBlock> ASTParser::parse_block() {
    expect(TokenType::LBRACE, "Expect 'do'");
    next();

    std::vector<std::unique_ptr<ASTNode>> statements;
    
    std::unique_ptr<ASTNode> stmt;
    while (pos < tokens.size() && !match(TokenType::RBRACE)) {
        stmt = parse_statement();
        
        if (stmt != nullptr) {
            statements.push_back(std::move(stmt));
        }
    }
    next();

    return std::make_unique<NodeBlock>(std::move(statements));
}

std::unique_ptr<NodeDeclare> ASTParser::parse_declare() {
    bool is_let = match(TokenType::LET);
    bool is_const = match(TokenType::CONST);

    if (!is_let && !is_const) {
        throw std::runtime_error("Expected 'let' or 'const'");
    }
    next();

    Token var_tkn = peek();
    next();

    expect(TokenType::COLON, "Expected ':'");
    next();

    auto type_node = parse_type();

    std::unique_ptr<NodeExpr> value_node = nullptr;
    if (!match(TokenType::TERMINATOR)) {
        expect(TokenType::ASSIGN, "Expected ':='");
        next();

        value_node = parse_expr();
    }

    return std::make_unique<NodeDeclare>(
        is_const,
        var_tkn.value,
        std::move(type_node),
        std::move(value_node)
    );
}

std::unique_ptr<NodeAssign> ASTParser::parse_assign() {
    std::string var_name = peek().value;
    next();
    next();
    auto value_node = parse_expr();

    return std::make_unique<NodeAssign>(
        var_name,
        std::move(value_node)
    );
}


std::unique_ptr<NodeExpr> ASTParser::parse_expr(int min_prec) {
    auto expr = parse_expr_primary();

    while (pos < tokens.size()) {
        Token op = peek();
        auto it = OP_PRECEDENCE.find(op.type);
        if (it == OP_PRECEDENCE.end()) {
            break;
        }
        int prec = OP_PRECEDENCE.at(op.type);
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
    auto tkn = peek();
    next();

    if (tkn.type == TokenType::LPAREN) {
        size_t save = pos;

        std::vector<std::string> args;
        if (!match(TokenType::RPAREN)) {
            do {
                expect(TokenType::IDENTIFIER, "Expected parameter name");
                args.push_back(peek().value);
                next();

                if (!match(TokenType::COMMA)) {
                    break;
                }
                next();
            } while (true);
        }
        expect(TokenType::RPAREN, "Expected ')'");
        next();

        if (match(TokenType::FATARROW)) {
            next();

            NodeLambda::BodyVariant body;
            if (match(TokenType::LBRACE)) {
                body = parse_block();
            } else {
                body = parse_expr();
            }

            return std::make_unique<NodeLambda>(args, std::move(body));
        }

        pos = save;

        std::vector<std::unique_ptr<NodeExpr>> elems;

        if (!match(TokenType::RPAREN)) {
            elems.push_back(parse_expr());

            while (match(TokenType::COMMA)) {
                next();
                elems.push_back(parse_expr());
            }
        }
        expect(TokenType::RPAREN, "Expected ')'");
        next();

        if (elems.size() > 1) {
            return std::make_unique<NodeTuple>(std::move(elems));
        }
        return std::move(elems[0]);
    }

    if (tkn.type == TokenType::IDENTIFIER) {
        if (match(TokenType::LPAREN)) {
            next();

            std::vector<std::unique_ptr<NodeExpr>> args;
            args.push_back(parse_expr());

            while (match(TokenType::COMMA)) {
                next();
                args.push_back(parse_expr());
            }
            expect(TokenType::RPAREN, "Expected ')'");
            next();

            return std::make_unique<NodeFunctionCall>(
                tkn.value,
                std::move(args)
            );
        }
        return std::make_unique<NodeVariable>(tkn.value);
    }

    std::stringstream ss(tkn.value);

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

    throw std::runtime_error("Unexpected token: " + tkn.value);
}

std::unique_ptr<NodeType> ASTParser::parse_type() {
    auto lhs = parse_product_type();

    while (pos < tokens.size() && peek().type == TokenType::ARROW) {
        next();
        auto rhs = parse_product_type();
        return std::make_unique<NodeMapType>(
            std::move(lhs),
            std::move(rhs)
        );
    }

    return lhs;
}

std::unique_ptr<NodeType> ASTParser::parse_product_type() {
    auto type = parse_sum_type();

    std::vector<std::unique_ptr<NodeType>> types;
    types.push_back(std::move(type));

    while (pos < tokens.size() && peek().type == TokenType::STAR) {
        next();
        types.push_back(parse_sum_type());
    }

    if (types.size() > 1) {
        return std::make_unique<NodeProductType>(std::move(types));
    }

    return std::move(types[0]);
}

std::unique_ptr<NodeType> ASTParser::parse_sum_type() {
    auto type = parse_type_primary();

    std::vector<std::unique_ptr<NodeType>> types;
    types.push_back(std::move(type));

    while (pos < tokens.size() && peek().type == TokenType::PLUS) {
        next();
        types.push_back(parse_type_primary());
    }

    if (types.size() > 1) {
        return std::make_unique<NodeProductType>(std::move(types));
    }

    return std::move(types[0]);
}

std::unique_ptr<NodeType> ASTParser::parse_type_primary() {
    auto tkn = peek();
    next();

    if (tkn.type == TokenType::LPAREN) {
        auto type = parse_type();
        expect(TokenType::RPAREN, "Expected ')'");
        next();
        return type;
    }

    if (tkn.type == TokenType::IDENTIFIER) {
        return std::make_unique<NodeAtomicType>(tkn.value);
    }

    return nullptr;
}

} // namespace fog