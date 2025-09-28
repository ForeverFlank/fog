#pragma once

#include "ast_nodes.h"

#include <unordered_map>
#include <utility>

namespace fog {

using ValueType = std::variant<
    int64_t,
    float,
    bool,
    std::string,
    std::vector<std::unique_ptr<Value>>
>;

struct Value {
    ValueType value;
    std::shared_ptr<Type> type;
};

struct Type : Value { };

struct PrimitiveType : Type {
    std::string name;
};

struct ProductType : Type {
    std::vector<std::shared_ptr<Type>> types;
};

struct SumType : Type {
    std::vector<std::shared_ptr<Type>> types;
};

struct MapType : Type {
    std::shared_ptr<Type> domain;
    std::shared_ptr<Type> codomain;
};

struct PrimitiveType {
    static std::shared_ptr<Type> Int;
};

class Scope {
public:
    Scope() : parent { nullptr } { }
    Scope(std::shared_ptr<Scope> parent) : parent{ parent } { }

    void init_var(std::string name, std::shared_ptr<Type> type);

    std::shared_ptr<Value> get_var(std::string name);
    void set_var(std::string name, std::shared_ptr<Value> value);
    
    std::shared_ptr<Type> get_atomic_type(std::string name);
    std::shared_ptr<Type> resolve_type(const NodeType *node);
    
private:
    std::shared_ptr<Scope> parent;
    std::unordered_map<std::string, std::shared_ptr<Value>> variables;
};

class Interpreter {
public:
    static std::shared_ptr<Value> eval(const ASTNode *node, std::shared_ptr<Scope> scope);
private:
    static std::shared_ptr<Value> eval_expr(const NodeExpr *node, std::shared_ptr<Scope> scope);
};

}  // namespace fog