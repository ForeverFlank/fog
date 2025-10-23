#pragma once

#include <memory>
#include <string>
#include <vector>
#include <variant>
#include <iostream>

namespace fog
{

struct ASTNode
{
    ASTNode() = default;
    virtual ~ASTNode() = default;

    virtual std::unique_ptr<ASTNode> clone() const = 0;

    virtual bool is_expression() const { return false; }
};

template<typename Derived>
static std::unique_ptr<Derived> downcast_unique_ptr(std::unique_ptr<ASTNode> p)
{
    return std::unique_ptr<Derived>(static_cast<Derived *>(p.release()));
}

struct NodeExpr : ASTNode
{
    std::unique_ptr<ASTNode> clone() const override
    {
        return std::make_unique<NodeExpr>(*this);
    }

    virtual void collect_used_variables(std::vector<std::string> &) const { };

    bool is_expression() const override { return true; }
};

struct NodeType : ASTNode
{
    std::unique_ptr<ASTNode> clone() const override
    {
        return std::make_unique<NodeType>(*this);
    }
};

struct NodeBlock : ASTNode
{
    std::vector<std::unique_ptr<ASTNode>> nodes;

    NodeBlock(std::vector<std::unique_ptr<ASTNode>> nodes)
        : nodes{std::move(nodes)}
    { }

    std::unique_ptr<ASTNode> clone() const override
    {
        std::vector<std::unique_ptr<ASTNode>> cloned;
        cloned.reserve(nodes.size());
        for (auto const &node : nodes)
        {
            cloned.push_back(node->clone());
        }
        return std::make_unique<NodeBlock>(std::move(cloned));
    }

    void collect_used_variables(std::vector<std::string> &out)
    {
        for (auto const &node : nodes)
        {
            if (auto expr = dynamic_cast<NodeExpr *>(node.get()))
            {
                expr->collect_used_variables(out);
            }
        }
    }
};

struct NodeMain : NodeBlock
{
    NodeMain(std::vector<std::unique_ptr<ASTNode>> nodes)
        : NodeBlock{std::move(nodes)}
    { }

    std::unique_ptr<ASTNode> clone() const override
    {
        std::vector<std::unique_ptr<ASTNode>> cloned;
        cloned.reserve(nodes.size());
        for (auto const &node : nodes)
        {
            cloned.push_back(node->clone());
        }
        return std::make_unique<NodeMain>(std::move(cloned));
    }
};

struct NodeDeclare : ASTNode
{
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
        type{std::move(type)}, value{std::move(value)}
    { }

    std::unique_ptr<ASTNode> clone() const override
    {
        return std::make_unique<NodeDeclare>(
            is_const,
            var_name,
            downcast_unique_ptr<NodeType>(type->clone()),
            downcast_unique_ptr<NodeExpr>(value->clone())
        );
    }
};

struct NodeAssign : ASTNode
{
    std::string var_name;
    std::unique_ptr<NodeExpr> value;

    NodeAssign(std::string var_name, std::unique_ptr<NodeExpr> value)
        : var_name{var_name}, value{std::move(value)}
    { }

    std::unique_ptr<ASTNode> clone() const override
    {
        return std::make_unique<NodeAssign>(
            var_name,
            downcast_unique_ptr<NodeExpr>(value->clone())
        );
    }
};

struct NodeReturn : ASTNode
{
    std::unique_ptr<NodeExpr> value;

    NodeReturn(std::unique_ptr<NodeExpr> value)
        : value{std::move(value)}
    { }

    std::unique_ptr<ASTNode> clone() const override
    {
        return std::make_unique<NodeReturn>(
            downcast_unique_ptr<NodeExpr>(value->clone())
        );
    }
};

struct NodeVariable : NodeExpr
{
    std::string name;

    NodeVariable(std::string name) : name{name} { }

    std::unique_ptr<ASTNode> clone() const override
    {
        return std::make_unique<NodeVariable>(name);
    }

    void collect_used_variables(std::vector<std::string> &out) const override
    {
        out.push_back(name);
    }
};

struct NodeLambda : NodeExpr
{
    std::vector<std::string> args;

    using BodyVariant = std::variant<
        std::unique_ptr<NodeBlock>,
        std::unique_ptr<NodeExpr>
    >;
    BodyVariant body;

    NodeLambda(std::vector<std::string> args, BodyVariant body)
        : args{args}, body{std::move(body)}
    { }

    std::unique_ptr<ASTNode> clone() const override
    {
        BodyVariant cloned;

        if (auto p = std::get_if<std::unique_ptr<NodeBlock>>(&body))
        {
            cloned = downcast_unique_ptr<NodeBlock>((*p)->clone());
        }
        else if (auto p = std::get_if<std::unique_ptr<NodeExpr>>(&body))
        {
            cloned = downcast_unique_ptr<NodeExpr>((*p)->clone());
        }
        return std::make_unique<NodeLambda>(args, std::move(cloned));
    }

    void collect_used_variables(std::vector<std::string> &out)
    {
        if (std::holds_alternative<std::unique_ptr<NodeBlock>>(body))
        {
            std::get<std::unique_ptr<NodeBlock>>(body)->collect_used_variables(out);
        }
        else if (std::holds_alternative<std::unique_ptr<NodeExpr>>(body))
        {
            std::get<std::unique_ptr<NodeExpr>>(body)->collect_used_variables(out);
        }
    }
};

struct NodeUnaryOp : NodeExpr
{
    std::string op;
    std::unique_ptr<NodeExpr> value;

    NodeUnaryOp(std::string op, std::unique_ptr<NodeExpr> value)
        : op{op}, value{std::move(value)}
    { }

    std::unique_ptr<ASTNode> clone() const override
    {
        return std::make_unique<NodeUnaryOp>(
            op,
            downcast_unique_ptr<NodeExpr>(value->clone())
        );
    }

    void collect_used_variables(std::vector<std::string> &out) const override
    {
        value->collect_used_variables(out);
    }
};

struct NodeBinaryOp : NodeExpr
{
    std::string op;
    std::unique_ptr<NodeExpr> lhs;
    std::unique_ptr<NodeExpr> rhs;

    NodeBinaryOp(std::string op,
        std::unique_ptr<NodeExpr> lhs,
        std::unique_ptr<NodeExpr> rhs)
        : op{op}, lhs{std::move(lhs)}, rhs{std::move(rhs)}
    { }

    std::unique_ptr<ASTNode> clone() const override
    {
        return std::make_unique<NodeBinaryOp>(
            op,
            downcast_unique_ptr<NodeExpr>(lhs->clone()),
            downcast_unique_ptr<NodeExpr>(rhs->clone())
        );
    }

    void collect_used_variables(std::vector<std::string> &out) const override
    {
        lhs->collect_used_variables(out);
        rhs->collect_used_variables(out);
    }
};

struct NodeTuple : NodeExpr
{
    std::vector<std::unique_ptr<NodeExpr>> elems;

    NodeTuple(std::vector<std::unique_ptr<NodeExpr>> elems)
        : elems{std::move(elems)}
    { }

    std::unique_ptr<ASTNode> clone() const override
    {
        std::vector<std::unique_ptr<NodeExpr>> cloned;
        cloned.reserve(elems.size());
        for (auto const &e : elems)
        {
            cloned.push_back(downcast_unique_ptr<NodeExpr>(e->clone()));
        }
        return std::make_unique<NodeTuple>(std::move(cloned));
    }

    void collect_used_variables(std::vector<std::string> &out) const override
    {
        for (auto const &elem : elems)
        {
            elem->collect_used_variables(out);
        }
    }
};

struct NodeFunctionCall : NodeExpr
{
    std::string name;
    std::vector<std::unique_ptr<NodeExpr>> args;

    NodeFunctionCall(std::string name,
        std::vector<std::unique_ptr<NodeExpr>> args)
        : name{name}, args{std::move(args)}
    { }

    std::unique_ptr<ASTNode> clone() const override
    {
        std::vector<std::unique_ptr<NodeExpr>> cloned;
        cloned.reserve(args.size());

        for (auto const &a : args)
        {
            cloned.push_back(downcast_unique_ptr<NodeExpr>(a->clone()));
        }

        return std::make_unique<NodeFunctionCall>(name, std::move(cloned));
    }
};

struct NodeInt32Literal : NodeExpr
{
    int32_t value;

    NodeInt32Literal(int32_t value) : value{value} { }

    std::unique_ptr<ASTNode> clone() const override
    {
        return std::make_unique<NodeInt32Literal>(value);
    }
};

struct NodeFloatLiteral : NodeExpr
{
    float value;

    NodeFloatLiteral(float value) : value{value} { }

    std::unique_ptr<ASTNode> clone() const override
    {
        return std::make_unique<NodeFloatLiteral>(value);
    }
};

struct NodeBoolLiteral : NodeExpr
{
    bool value;

    NodeBoolLiteral(bool value) : value{value} { }

    std::unique_ptr<ASTNode> clone() const override
    {
        return std::make_unique<NodeBoolLiteral>(value);
    }
};

struct NodeCharLiteral : NodeExpr
{
    char value;

    NodeCharLiteral(char value) : value{value} { }

    std::unique_ptr<ASTNode> clone() const override
    {
        return std::make_unique<NodeCharLiteral>(value);
    }
};

struct NodeStringLiteral : NodeExpr
{
    std::string value;

    NodeStringLiteral(std::string value) : value{std::move(value)} { }

    std::unique_ptr<ASTNode> clone() const override
    {
        return std::make_unique<NodeStringLiteral>(value);
    }
};

struct NodeAtomicType : NodeType
{
    std::string name;

    NodeAtomicType(std::string name) : name{std::move(name)} { }

    std::unique_ptr<ASTNode> clone() const override
    {
        return std::make_unique<NodeAtomicType>(name);
    }
};

struct NodeSumType : NodeType
{
    std::vector<std::unique_ptr<NodeType>> types;

    NodeSumType(std::vector<std::unique_ptr<NodeType>> types)
        : types{std::move(types)}
    { }

    std::unique_ptr<ASTNode> clone() const override
    {
        std::vector<std::unique_ptr<NodeType>> cloned;
        cloned.reserve(types.size());
        for (auto const &t : types)
        {
            cloned.push_back(downcast_unique_ptr<NodeType>(t->clone()));
        }
        return std::make_unique<NodeSumType>(std::move(cloned));
    }
};

struct NodeProductType : NodeType
{
    std::vector<std::unique_ptr<NodeType>> types;

    NodeProductType(std::vector<std::unique_ptr<NodeType>> types)
        : types{std::move(types)}
    { }

    std::unique_ptr<ASTNode> clone() const override
    {
        std::vector<std::unique_ptr<NodeType>> cloned;
        cloned.reserve(types.size());
        for (auto const &t : types)
        {
            cloned.push_back(downcast_unique_ptr<NodeType>(t->clone()));
        }
        return std::make_unique<NodeProductType>(std::move(cloned));
    }
};

struct NodeMapType : NodeType
{
    std::unique_ptr<NodeType> domain;
    std::unique_ptr<NodeType> codomain;

    NodeMapType(std::unique_ptr<NodeType> domain,
        std::unique_ptr<NodeType> codomain)
        : domain{std::move(domain)}, codomain{std::move(codomain)}
    { }

    std::unique_ptr<ASTNode> clone() const override
    {
        return std::make_unique<NodeMapType>(
            downcast_unique_ptr<NodeType>(domain->clone()),
            downcast_unique_ptr<NodeType>(codomain->clone())
        );
    }
};

} // namespace fog
