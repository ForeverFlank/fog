#pragma once

#include <memory>
#include <string>
#include <vector>
#include <variant>

namespace fog {

struct ASTNode {
    virtual ~ASTNode() = default;
};

struct NodeExpr : ASTNode { };

struct NodeType : ASTNode { };

struct NodeBlock : ASTNode {
    std::vector<std::unique_ptr<ASTNode>> nodes;

    NodeBlock(
        std::vector<std::unique_ptr<ASTNode>> nodes
    ) : nodes{std::move(nodes)} { }
};

struct NodeMain : NodeBlock {
    NodeMain(
        std::vector<std::unique_ptr<ASTNode>> nodes
    ) : NodeBlock{std::move(nodes)} { }
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
    ) : is_const{is_const}, var_name{var_name},
        type{std::move(type)}, value{std::move(value)} { }
};

struct NodeAssign : ASTNode {
    std::string var_name;
    std::unique_ptr<NodeExpr> value;

    NodeAssign(
        std::string var_name,
        std::unique_ptr<NodeExpr> value
    ) : var_name{var_name}, value{std::move(value)} { }
};

struct NodeReturn : ASTNode {
    std::unique_ptr<NodeExpr> value;

    NodeReturn(
        std::unique_ptr<NodeExpr> value
    ) : value{std::move(value)} { }
};

struct NodeVariable : NodeExpr {
    std::string name;

    NodeVariable(std::string name) : name{name} { }
};

struct NodeLambda : NodeExpr {
    std::vector<std::string> args;

    using BodyVariant = std::variant<
        std::unique_ptr<NodeBlock>,
        std::unique_ptr<NodeExpr>
    >;

    BodyVariant body;

    NodeLambda(
        std::vector<std::string> args, BodyVariant body
    ) : args{args}, body{std::move(body)} { }
};

struct NodeUnaryOp : NodeExpr {
    std::string op;
    std::unique_ptr<NodeExpr> value;

    NodeUnaryOp(
        std::string op,
        std::unique_ptr<NodeExpr> value
    ) : op{op}, value{std::move(value)} { }
};

struct NodeBinaryOp : NodeExpr {
    std::string op;
    std::unique_ptr<NodeExpr> lhs;
    std::unique_ptr<NodeExpr> rhs;

    NodeBinaryOp(
        std::string op,
        std::unique_ptr<NodeExpr> lhs,
        std::unique_ptr<NodeExpr> rhs
    ) : op{op}, lhs{std::move(lhs)}, rhs{std::move(rhs)} { }
};

struct NodeTuple : NodeExpr {
    std::vector<std::unique_ptr<NodeExpr>> elems;

    NodeTuple(
        std::vector<std::unique_ptr<NodeExpr>> elems
    ) : elems{std::move(elems)} { }
};

struct NodeFunctionCall : NodeExpr {
    std::string name;
    std::vector<std::unique_ptr<NodeExpr>> args;

    NodeFunctionCall(
        std::string function_name,
        std::vector<std::unique_ptr<NodeExpr>> args
    ) : name{function_name}, args{std::move(args)} { }
};

struct NodeInt32Literal : NodeExpr {
    int32_t value;

    NodeInt32Literal(int32_t value) : value{value} { }
};

struct NodeFloatLiteral : NodeExpr {
    float value;

    NodeFloatLiteral(float value) : value{value} { }
};

struct NodeBoolLiteral : NodeExpr {
    bool value;

    NodeBoolLiteral(bool value) : value{value} { }
};

struct NodeDoubleLiteral : NodeExpr {
    double value;

    NodeDoubleLiteral(double value) : value{value} { }
};

struct NodeCharLiteral : NodeExpr {
    char value;
};

struct NodeStringLiteral : NodeExpr {
    std::string value;
};

struct NodeAtomicType : NodeType {
    std::string name;

    NodeAtomicType(std::string name) : name{name} { }
};

struct NodeSumType : NodeType {
    std::vector<std::unique_ptr<NodeType>> types;

    NodeSumType(
        std::vector<std::unique_ptr<NodeType>> types
    ) : types{std::move(types)} { }
};

struct NodeProductType : NodeType {
    std::vector<std::unique_ptr<NodeType>> types;

    NodeProductType(
        std::vector<std::unique_ptr<NodeType>> types
    ) : types{std::move(types)} { }
};

struct NodeMapType : NodeType {
    std::unique_ptr<NodeType> domain;
    std::unique_ptr<NodeType> codomain;

    NodeMapType(
        std::unique_ptr<NodeType> domain,
        std::unique_ptr<NodeType> codomain
    ) : domain{std::move(domain)},
        codomain{std::move(codomain)} { }
};

}  // namespace fog