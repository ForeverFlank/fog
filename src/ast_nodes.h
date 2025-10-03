#pragma once

#include <memory>
#include <string>
#include <vector>
#include <variant>

namespace fog {

struct ASTNode {
    ASTNode() { }
    virtual ~ASTNode() = default;
    
    std::unique_ptr<ASTNode> clone() {
        return std::unique_ptr<ASTNode>();
    }
};

struct NodeExpr : ASTNode {
    NodeExpr() { }
    
    std::unique_ptr<NodeExpr> clone() {
        return std::unique_ptr<NodeExpr>();
    }
};

struct NodeType : ASTNode {
    NodeType() { }

    std::unique_ptr<NodeType> clone() {
        return std::unique_ptr<NodeType>();
    }
};

struct NodeBlock : ASTNode {
    std::vector<std::unique_ptr<ASTNode>> nodes;

    NodeBlock(
        std::vector<std::unique_ptr<ASTNode>> nodes
    ) : nodes{std::move(nodes)} { }
    
    std::unique_ptr<NodeBlock> clone() {
        std::vector<std::unique_ptr<ASTNode>> cloned;

        for (size_t i = 0; i < nodes.size(); i++) {
            cloned.push_back(nodes[i]->clone());
        }

        return std::make_unique<NodeBlock>(std::move(cloned));
    }
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
    
    std::unique_ptr<NodeDeclare> clone() {
        return std::make_unique<NodeDeclare>(
            is_const,
            var_name,
            std::move(type->clone()),
            std::move(value->clone())
        );
    }
};

struct NodeAssign : ASTNode {
    std::string var_name;
    std::unique_ptr<NodeExpr> value;

    NodeAssign(
        std::string var_name,
        std::unique_ptr<NodeExpr> value
    ) : var_name{var_name}, value{std::move(value)} { }

    std::unique_ptr<NodeAssign> clone() {
        return std::make_unique<NodeAssign>(
            var_name,
            std::move(value->clone())
        );
    }
};

struct NodeReturn : ASTNode {
    std::unique_ptr<NodeExpr> value;

    NodeReturn(
        std::unique_ptr<NodeExpr> value
    ) : value{std::move(value)} { }

    std::unique_ptr<NodeReturn> clone() {
        return std::make_unique<NodeReturn>(std::move(value->clone()));
    }
};

struct NodeVariable : NodeExpr {
    std::string name;
    
    NodeVariable(std::string name) : name{name} { }

    std::unique_ptr<NodeVariable> clone() {
        return std::make_unique<NodeVariable>(name);
    }
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

    std::unique_ptr<NodeLambda> clone() const {
        BodyVariant cloned;

        if (std::holds_alternative<std::unique_ptr<NodeBlock>>(body)) {
            cloned = std::get<std::unique_ptr<NodeBlock>>(body)->clone();
        } else {
            cloned = std::get<std::unique_ptr<NodeExpr>>(body)->clone();
        }

        return std::make_unique<NodeLambda>(args, std::move(cloned));
    }
};

struct NodeUnaryOp : NodeExpr {
    std::string op;
    std::unique_ptr<NodeExpr> value;

    NodeUnaryOp(
        std::string op,
        std::unique_ptr<NodeExpr> value
    ) : op{op}, value{std::move(value)} { }

    std::unique_ptr<NodeUnaryOp> clone() {
        return std::make_unique<NodeUnaryOp>(
            op,
            std::move(value->clone())
        );
    }
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

    std::unique_ptr<NodeBinaryOp> clone() {
        return std::make_unique<NodeBinaryOp>(
            op,
            std::move(lhs->clone()),
            std::move(rhs->clone())
        );
    }
};

struct NodeTuple : NodeExpr {
    std::vector<std::unique_ptr<NodeExpr>> elems;

    NodeTuple(
        std::vector<std::unique_ptr<NodeExpr>> elems
    ) : elems{std::move(elems)} { }

    std::unique_ptr<NodeTuple> clone() {
        std::vector<std::unique_ptr<NodeExpr>> cloned;

        for (size_t i = 0; i < elems.size(); i++) {
            cloned.push_back(std::move(elems[i]->clone()));
        }

        return std::make_unique<NodeTuple>(std::move(cloned));
    }
};

struct NodeFunctionCall : NodeExpr {
    std::string name;
    std::vector<std::unique_ptr<NodeExpr>> args;

    NodeFunctionCall(
        std::string function_name,
        std::vector<std::unique_ptr<NodeExpr>> args
    ) : name{function_name}, args{std::move(args)} { }

    std::unique_ptr<NodeFunctionCall> clone() {
        std::vector<std::unique_ptr<NodeExpr>> cloned;

        for (size_t i = 0; i < args.size(); i++) {
            cloned.push_back(std::move(args[i]->clone()));
        }

        return std::make_unique<NodeFunctionCall>(name, std::move(cloned));
    }
};

struct NodeInt32Literal : NodeExpr {
    int32_t value;

    NodeInt32Literal(int32_t value) : value{value} { }

    std::unique_ptr<NodeInt32Literal> clone() {
        return std::make_unique<NodeInt32Literal>(value);
    }
};

struct NodeFloatLiteral : NodeExpr {
    float value;

    NodeFloatLiteral(float value) : value{value} { }

    std::unique_ptr<NodeFloatLiteral> clone() {
        return std::make_unique<NodeFloatLiteral>(value);
    }
};

struct NodeBoolLiteral : NodeExpr {
    bool value;

    NodeBoolLiteral(bool value) : value{value} { }

    std::unique_ptr<NodeBoolLiteral> clone() {
        return std::make_unique<NodeBoolLiteral>(value);
    }
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

    std::unique_ptr<NodeAtomicType> clone() {
        return std::make_unique<NodeAtomicType>(name);
    }
};

struct NodeSumType : NodeType {
    std::vector<std::unique_ptr<NodeType>> types;

    NodeSumType(
        std::vector<std::unique_ptr<NodeType>> types
    ) : types{std::move(types)} { }

    std::unique_ptr<NodeSumType> clone() {
        std::vector<std::unique_ptr<NodeType>> cloned;

        for (size_t i = 0; i < types.size(); i++) {
            cloned.push_back(std::move(types[i]->clone()));
        }

        return std::make_unique<NodeSumType>(std::move(cloned));
    }
};

struct NodeProductType : NodeType {
    std::vector<std::unique_ptr<NodeType>> types;

    NodeProductType(
        std::vector<std::unique_ptr<NodeType>> types
    ) : types{std::move(types)} { }

    std::unique_ptr<NodeProductType> clone() {
        std::vector<std::unique_ptr<NodeType>> cloned;

        for (size_t i = 0; i < types.size(); i++) {
            cloned.push_back(std::move(types[i]->clone()));
        }

        return std::make_unique<NodeProductType>(std::move(cloned));
    }
};

struct NodeMapType : NodeType {
    std::unique_ptr<NodeType> domain;
    std::unique_ptr<NodeType> codomain;

    NodeMapType(
        std::unique_ptr<NodeType> domain,
        std::unique_ptr<NodeType> codomain
    ) : domain{std::move(domain)},
        codomain{std::move(codomain)} { }
    
    std::unique_ptr<NodeMapType> clone() {
        return std::make_unique<NodeMapType>(
            std::move(domain->clone()),
            std::move(codomain->clone())
        );
    }
};

}  // namespace fog