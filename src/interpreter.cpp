#include "interpreter.h"

#include <stdexcept>

namespace fog {

void Scope::init_var(std::string name, std::shared_ptr<Type> type) {
    auto value = std::make_shared<Value>();
    value->type = type;

    variables[name] = value;
}

std::shared_ptr<Value> Scope::get_var(std::string name) {
    auto it = variables.find(name);
    if (it != variables.end()) {
        return it->second;
    }
    if (parent != nullptr) {
        return parent->get_var(name);
    }
    throw std::runtime_error("Undefined variable: " + name);
}

void Scope::set_var(std::string name, std::shared_ptr<Value> value) {
    if (variables.contains(name)) {
        variables[name] = value;
        return;
    }
    throw std::runtime_error("Undefined variable: " + name);
}

BinaryOpFunction Scope::get_op(BinaryOpKey key) {
    auto it = operators.find(key);
    if (it != operators.end()) {
        return it->second;
    }
    if (parent != nullptr) {
        return parent->get_op(key);
    }
    throw std::runtime_error("Undefined operator: " + std::get<0>(key));
}

void Scope::set_op(BinaryOpKey key, BinaryOpFunction value) {
    operators[key] = value;
}

std::shared_ptr<Type> Scope::get_atomic_type(std::string name) {
    auto value = get_var(name);
    return std::dynamic_pointer_cast<Type>(value);
}

std::shared_ptr<Type> Scope::resolve_type(const NodeType *node) {
    if (auto casted = dynamic_cast<const NodeAtomicType *>(node)) {
        return get_atomic_type(casted->name);
    }

    if (auto casted = dynamic_cast<const NodeProductType *>(node)) {
        auto res = std::make_shared<ProductType>();
        for (auto& tkn : casted->types) {
            auto type = resolve_type(tkn.get());
            res->types.push_back(type);
        }
        return res;
    }

    if (auto casted = dynamic_cast<const NodeSumType *>(node)) {
        auto res = std::make_shared<SumType>();
        for (auto& tkn : casted->types) {
            auto type = resolve_type(tkn.get());
            res->types.push_back(type);
        }
        return res;
    }

    if (auto casted = dynamic_cast<const NodeMapType *>(node)) {
        auto res = std::make_shared<MapType>();
        res->domain = resolve_type(casted->domain.get());
        res->codomain = resolve_type(casted->codomain.get());
        return res;
    }
}

std::shared_ptr<Value> Interpreter::eval(
    const ASTNode *node,
    std::shared_ptr<Scope> scope
) {
    if (auto casted = dynamic_cast<const NodeMain *>(node)) {
        for (auto &stmt : casted->nodes) {
            eval(stmt.get(), scope);
        }
        return nullptr;
    }

    if (auto casted = dynamic_cast<const NodeBlock *>(node)) {
        auto block_scope = std::make_shared<Scope>(scope);
        for (auto &stmt : casted->nodes) {
            eval(stmt.get(), block_scope);
        }
        return nullptr;
    }

    if (auto casted = dynamic_cast<const NodeDeclare *>(node)) {
        std::string name = casted->var_name;
        auto type = scope->resolve_type(casted->type.get());
        scope->init_var(name, type);
        
        if (casted->value != nullptr) {
            auto value = eval(casted->value.get(), scope);
            scope->set_var(name, value);
        }
        return nullptr;
    }
    
    if (auto casted = dynamic_cast<const NodeAssign *>(node)) {
        std::string name = casted->var_name;
        auto value = eval(casted->value.get(), scope);
        scope->set_var(name, value);
        return nullptr;
    }

    if (auto casted = dynamic_cast<const NodeExpr *>(node)) {
        return eval_expr(casted, scope);
    }

    if (auto casted = dynamic_cast<const NodeReturn *>(node)) {

    }

    return nullptr;
}

std::shared_ptr<Value> Interpreter::eval_expr(
    const NodeExpr *node,
    std::shared_ptr<Scope> scope
) {
    if (auto casted = dynamic_cast<const NodeVariable *>(node)) {
        return scope->get_var(casted->name);
    }

    // if (auto casted = dynamic_cast<const NodeLambda *>(node)) {
    //     return std::make_shared<Value>(
    //         casted->args,
    //         scope->resolve_type(casted->???)
    //     )
    // }

    if (auto casted = dynamic_cast<const NodeBinaryOp *>(node)) {
        auto lhs = eval(casted->lhs.get(), scope);
        auto rhs = eval(casted->rhs.get(), scope);

        auto key = std::make_tuple(casted->op, lhs->type, rhs->type);
        auto op = scope->get_op(key);

        auto res = op(lhs, rhs);

        return res;
    }

    if (auto casted = dynamic_cast<const NodeTuple *>(node)) {
        
    }

    if (auto casted = dynamic_cast<const NodeFunctionCall *>(node)) {
        
    }

    if (auto casted = dynamic_cast<const NodeInt64Literal *>(node)) {
        return std::make_shared<Value>(
            casted->value,
            scope->get_atomic_type("int")
        );
    }
    
    if (auto casted = dynamic_cast<const NodeFloatLiteral *>(node)) {
        return std::make_shared<Value>(
            casted->value,
            scope->get_atomic_type("float")
        );
    }
    
    if (auto casted = dynamic_cast<const NodeBoolLiteral *>(node)) {
        return std::make_shared<Value>(
            casted->value,
            scope->get_atomic_type("bool")
        );
    }
}

}