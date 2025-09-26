#pragma once

#include <memory>
#include <string>
#include <vector>

namespace fog {

struct ASTNode {
    virtual ~ASTNode() = default;
};

struct NodeType : ASTNode { };

struct NodeExpr : ASTNode { };

struct NodePrimitiveType : NodeType {
    std::string name;

    NodePrimitiveType(std::string name) : name{name} { };
};

struct NodeTupleType : NodeType {
    std::vector<std::unique_ptr<NodePrimitiveType>> types;
};

struct NodeMapType : NodeType {
    std::unique_ptr<NodeType> domain;
    std::unique_ptr<NodeType> codomain;
};

struct NodeVariable : ASTNode {
    std::string name;
    std::unique_ptr<NodeType> type;

    NodeVariable(
        std::string name,
        std::unique_ptr<NodeType> type
    ) : name{name}, type{std::move(type)} { };
};

struct NodeBlock : ASTNode {
    std::vector<std::unique_ptr<ASTNode>> nodes;

    NodeBlock(
        std::vector<std::unique_ptr<ASTNode>> nodes
    ) : nodes{std::move(nodes)} { }
};

struct NodeDeclare : ASTNode {
    bool is_const = false;
    std::unique_ptr<NodeVariable> var;
    std::unique_ptr<NodeExpr> value;

    NodeDeclare(
        bool is_const,
        std::unique_ptr<NodeVariable> var,
        std::unique_ptr<NodeExpr> value
    ) : is_const{is_const}, var{std::move(var)}, value{std::move(value)} { }
};

struct NodeAssign : ASTNode {
    std::unique_ptr<NodeVariable> var;
    std::unique_ptr<NodeExpr> value;

    NodeAssign(
        std::unique_ptr<NodeVariable> var,
        std::unique_ptr<NodeExpr> value
    ) : var{std::move(var)}, value{std::move(value)} { }
};

struct NodeBinaryOp : NodeExpr {
    std::string op;
    std::unique_ptr<NodeExpr> lhs;
    std::unique_ptr<NodeExpr> rhs;

    NodeBinaryOp(
        std::string op,
        std::unique_ptr<NodeExpr> lhs,
        std::unique_ptr<NodeExpr> rhs
    ) : op{op}, lhs{std::move(lhs)}, rhs{std::move(rhs)} { };
};

struct NodeInt64Literal : NodeExpr {
    int64_t value;

    NodeInt64Literal(int64_t value) : value{value} { };
};

struct NodeFloatLiteral : NodeExpr {
    float value;

    NodeFloatLiteral(float value) : value{value} { };
};

struct NodeDoubleLiteral : NodeExpr {
    double value;

    NodeDoubleLiteral(double value) : value{value} { };
};

struct NodeCharLiteral : NodeExpr {
    char value;
};

struct NodeStringLiteral : NodeExpr {
    std::string value;
};

}  // namespace fog