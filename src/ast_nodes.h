#pragma once

#include <memory>
#include <string>
#include <vector>

namespace fog {

struct ASTNode {
    virtual ~ASTNode() = default;
};

struct NodeExpr : ASTNode { };

struct NodeType : ASTNode { };

struct NodeAtomicType : NodeType {
    std::string name;

    NodeAtomicType(std::string name) : name{ name } { }
};

struct NodeSumType : NodeType {
    std::vector<std::unique_ptr<NodeType>> types;

    NodeSumType(
        std::vector<std::unique_ptr<NodeType>> types
    ) : types{ std::move(types) } { }
};

struct NodeProductType : NodeType {
    std::vector<std::unique_ptr<NodeType>> types;

    NodeProductType(
        std::vector<std::unique_ptr<NodeType>> types
    ) : types{ std::move(types) } { }
};

struct NodeMapType : NodeType {
    std::unique_ptr<NodeType> domain;
    std::unique_ptr<NodeType> codomain;

    NodeMapType(
        std::unique_ptr<NodeType> domain,
        std::unique_ptr<NodeType> codomain
    ) : domain{ std::move(domain) }, codomain{ std::move(codomain) } { }
};

struct NodeVariable : NodeExpr {
    std::string name;

    NodeVariable(std::string name) : name{ name } { }
};

struct NodeBlock : ASTNode {
    std::vector<std::unique_ptr<ASTNode>> nodes;

    NodeBlock(
        std::vector<std::unique_ptr<ASTNode>> nodes
    ) : nodes{ std::move(nodes) } { }
};

struct NodeDeclare : ASTNode {
    bool is_const = false;
    std::string var_name;
    std::unique_ptr<NodeType> type;
    std::unique_ptr<NodeExpr> value;

    NodeDeclare(
        bool is_const,
        std::string var_name,
        std::unique_ptr<NodeType> type,
        std::unique_ptr<NodeExpr> value
    ) : is_const{ is_const }, var_name{ var_name }, type{ std::move(type) }, value{ std::move(value) } { }
};

struct NodeAssign : ASTNode {
    std::string var_name;
    std::unique_ptr<NodeExpr> value;

    NodeAssign(
        std::string var_name,
        std::unique_ptr<NodeExpr> value
    ) : var_name{ var_name }, value{ std::move(value) } { }
};

struct NodeBinaryOp : NodeExpr {
    std::string op;
    std::unique_ptr<NodeExpr> lhs;
    std::unique_ptr<NodeExpr> rhs;

    NodeBinaryOp(
        std::string op,
        std::unique_ptr<NodeExpr> lhs,
        std::unique_ptr<NodeExpr> rhs
    ) : op{ op }, lhs{ std::move(lhs) }, rhs{ std::move(rhs) } { }
};

struct NodeInt64Literal : NodeExpr {
    int64_t value;

    NodeInt64Literal(int64_t value) : value{ value } { }
};

struct NodeFloatLiteral : NodeExpr {
    float value;

    NodeFloatLiteral(float value) : value{ value } { }
};

struct NodeDoubleLiteral : NodeExpr {
    double value;

    NodeDoubleLiteral(double value) : value{ value } { }
};

struct NodeCharLiteral : NodeExpr {
    char value;
};

struct NodeStringLiteral : NodeExpr {
    std::string value;
};

}  // namespace fog