#pragma once

#include "ast_nodes.h"

#include <functional>
#include <unordered_map>
#include <utility>
#include <variant>

namespace fog {

struct Type;

struct Value {
    using ValueType = std::variant<
        int32_t,
        float,
        bool,
        std::string,
        std::vector<std::unique_ptr<Value>>
    >;

    ValueType value;
    std::shared_ptr<Type> type;

    Value() = default;
    Value(
        std::shared_ptr<Type> type
    ) : type{type} { };
    Value(
        ValueType value,
        std::shared_ptr<Type> type
    ) : value{std::move(value)}, type{type} { };

    virtual ~Value() = default;
};

struct Type : virtual Value {
    Type() { }
};

using BinaryOpKey = std::tuple<
    std::string,
    std::shared_ptr<Type>,
    std::shared_ptr<Type>
>;

using BinaryOpFunction = std::function<
    std::shared_ptr<Value>(std::shared_ptr<Value>, std::shared_ptr<Value>)
>;

struct BinaryOpKeyHash {
    size_t operator()(const BinaryOpKey &key) const {
        return
            std::hash<std::string>()(std::get<0>(key)) ^
            std::hash<Type *>()(std::get<1>(key).get()) ^
            std::hash<Type *>()(std::get<2>(key).get());
    }
};

struct PrimitiveType : Type {
    std::string name;

    PrimitiveType(std::string name) : name{name} { }
    PrimitiveType(std::string name, std::shared_ptr<Type> type)
        : Value(type), name{name} { }
};

struct ProductType : Type {
    std::vector<std::shared_ptr<Type>> types;

    ProductType() = default;
    ProductType(std::vector<std::shared_ptr<Type>> types)
        : Type{ }, types{std::move(types)} { }
};

struct SumType : Type {
    std::vector<std::shared_ptr<Type>> types;

    SumType() = default;
    SumType(std::vector<std::shared_ptr<Type>> types)
        : Type{ }, types{std::move(types)} { }
};

struct MapType : Type {
    std::shared_ptr<Type> domain;
    std::shared_ptr<Type> codomain;

    MapType() = default;
    MapType(std::shared_ptr<Type> domain, std::shared_ptr<Type> codomain)
        : Type{ }, domain{std::move(domain)}, codomain{std::move(codomain)} { }
};

class Scope {
public:
    Scope() : parent{nullptr} { }
    Scope(std::shared_ptr<Scope> parent) : parent{parent} { }

    void init_var(std::string name, std::shared_ptr<Type> type);
    
    std::shared_ptr<Value> get_var(std::string name);
    void                   set_var(std::string name, std::shared_ptr<Value> value);

    BinaryOpFunction get_op(BinaryOpKey key);
    void             set_op(BinaryOpKey key, BinaryOpFunction value);

    std::shared_ptr<Type> get_atomic_type(std::string name);
    std::shared_ptr<Type> resolve_type(const NodeType *node);

    // private:
    std::shared_ptr<Scope> parent;

    std::unordered_map<std::string, std::shared_ptr<Value>> variables;
    std::unordered_map<
        BinaryOpKey,
        BinaryOpFunction,
        BinaryOpKeyHash
    > operators;
};

class Interpreter {
public:
    std::shared_ptr<Scope> global_scope;

    Interpreter();

    std::shared_ptr<Value> eval(const ASTNode *node) {
        return eval(node, global_scope);
    }

    static std::shared_ptr<Value> eval(const ASTNode *node, std::shared_ptr<Scope> scope);

private:
    static std::shared_ptr<Value> eval_expr(const NodeExpr *node, std::shared_ptr<Scope> scope);
};

}  // namespace fog