#pragma once

#include <memory>
#include <string>
#include <vector>

namespace fog {

struct ASTNode {
    virtual ~ASTNode() = default;
};

struct NodeBlock : ASTNode {
    std::vector<std::unique_ptr<ASTNode>> nodes;
};

struct NodeDeclaration : ASTNode {
    bool is_const = false;
    std::unique_ptr<NodeVariable> var;
    std::unique_ptr<NodeType> type;
    std::unique_ptr<NodeExpr> value;
};

struct NodeAssignment : ASTNode {
    std::unique_ptr<NodeVariable> var;
    std::unique_ptr<NodeExpr> value;
};

struct NodeType : ASTNode {};

struct NodePrimitiveType : NodeType {
    std::string name;
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
};

struct NodeExpr : ASTNode {};

struct NodeBinaryOp : NodeExpr {
    std::string op;
    std::unique_ptr<NodeExpr> left;
    std::unique_ptr<NodeExpr> right;
};

struct NodeInt32Literal : NodeExpr {
    int32_t value;
};

struct NodeInt64Literal : NodeExpr {
    int64_t value;
};

struct NodeFloatLiteral : NodeExpr {
    float value;
};

struct NodeDoubleLiteral : NodeExpr {
    double value;
};

struct NodeCharLiteral : NodeExpr {
    char value;
};

struct NodeStringLiteral : NodeExpr {
    std::string value;
};

}  // namespace fog