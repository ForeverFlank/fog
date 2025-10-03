#include "interpreter.h"

#include <iostream>
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
        for (auto &tkn : casted->types) {
            auto type = resolve_type(tkn.get());
            res->types.push_back(type);
        }
        return res;
    }

    if (auto casted = dynamic_cast<const NodeSumType *>(node)) {
        auto res = std::make_shared<SumType>();
        for (auto &tkn : casted->types) {
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

    throw std::runtime_error("Unexpected node");
}

Interpreter::Interpreter() {
    global_scope = std::make_shared<Scope>();

    auto type_type = std::make_shared<PrimitiveType>("type");
    type_type->type = type_type;
    global_scope->init_var("type", type_type);
    global_scope->set_var("type", type_type);

    auto init_type = [this, type_type] (std::string name) {
        auto type = std::make_shared<PrimitiveType>(name, type_type);
        global_scope->init_var(name, type_type);
        global_scope->set_var(name, type);
        };

    init_type("int");
    init_type("float");
    init_type("bool");
    init_type("lambda");

    auto int_type = global_scope->get_atomic_type("int");
    auto float_type = global_scope->get_atomic_type("float");

    auto make_int_op = [this, int_type] (auto op) {
        return [this, op, int_type] (std::shared_ptr<Value> a, std::shared_ptr<Value> b) {
            int32_t a_val = std::get<int32_t>(a->value);
            int32_t b_val = std::get<int32_t>(b->value);
            auto result = op(a_val, b_val);
            return std::make_shared<Value>(result, int_type);
            };
        };

    auto make_float_op = [this, float_type, int_type] (auto op) {
        return [this, op, float_type, int_type] (std::shared_ptr<Value> a, std::shared_ptr<Value> b) {
            float a_val, b_val;

            a_val = (a->type == int_type)
                ? static_cast<float>(std::get<int32_t>(a->value))
                : std::get<float>(a->value);

            b_val = (b->type == int_type)
                ? static_cast<float>(std::get<int32_t>(b->value))
                : std::get<float>(b->value);

            auto result = op(a_val, b_val);
            return std::make_shared<Value>(result, float_type);
            };
        };

    global_scope->set_op({"+", int_type, int_type},
        make_int_op([] (int32_t a, int32_t b) { return a + b; })
    );
    global_scope->set_op({"-", int_type, int_type},
        make_int_op([] (int32_t a, int32_t b) { return a - b; })
    );
    global_scope->set_op({"*", int_type, int_type},
        make_int_op([] (int32_t a, int32_t b) { return a * b; })
    );
    global_scope->set_op({"div", int_type, int_type},
        make_int_op([] (int32_t a, int32_t b) { return a / b; })
    );
    global_scope->set_op({"mod", int_type, int_type},
        make_int_op([] (int32_t a, int32_t b) { return a % b; })
    );
    global_scope->set_op({"-", nullptr, int_type},
        [this, int_type] (std::shared_ptr<Value> a, std::shared_ptr<Value> b) {
            return std::make_shared<Value>(
                -std::get<int32_t>(b->value),
                int_type);
        }
    );

    for (int i = 1; i < 4; i++) {
        auto type_a = (i & 1) ? float_type : int_type;
        auto type_b = (i & 2) ? float_type : int_type;

        global_scope->set_op({"+", type_a, type_b},
            make_float_op([] (float a, float b) { return a + b; })
        );
        global_scope->set_op({"-", type_a, type_b},
            make_float_op([] (float a, float b) { return a - b; })
        );
        global_scope->set_op({"*", type_a, type_b},
            make_float_op([] (float a, float b) { return a * b; })
        );
        global_scope->set_op({"/", type_a, type_b},
            make_float_op([] (float a, float b) { return a / b; })
        );
    }
    global_scope->set_op({"-", nullptr, float_type},
        [this, float_type] (std::shared_ptr<Value>, std::shared_ptr<Value> b) {
            return std::make_shared<Value>(
                -std::get<float>(b->value),
                float_type);
        }
    );
}

std::shared_ptr<ReturnValue> Interpreter::eval(
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
            auto res = eval(stmt.get(), block_scope);
            if (res != nullptr) {
                return res;
            }
        }
        return nullptr;
    }

    if (auto casted = dynamic_cast<const NodeDeclare *>(node)) {
        std::string name = casted->var_name;
        auto type = scope->resolve_type(casted->type.get());
        scope->init_var(name, type);

        if (casted->value != nullptr) {
            auto value = eval_expr(casted->value.get(), scope);
            scope->set_var(name, value);
        }
        return nullptr;
    }

    if (auto casted = dynamic_cast<const NodeAssign *>(node)) {
        std::string name = casted->var_name;
        auto value = eval_expr(casted->value.get(), scope);
        scope->set_var(name, value);
        return nullptr;
    }

    // if (auto casted = dynamic_cast<const NodeExpr *>(node)) {
    //     return eval_expr(casted, scope);
    // }

    if (auto casted = dynamic_cast<const NodeReturn *>(node)) {
        auto value = eval_expr(casted->value.get(), scope);
        return std::make_shared<ReturnValue>(value);
    }

    throw std::runtime_error("Unexpected node");
}

std::shared_ptr<Value> Interpreter::eval_expr(
    const NodeExpr *node,
    std::shared_ptr<Scope> scope
) {
    if (node == nullptr) {
        throw std::runtime_error("Null expression pointer");
    }

    if (auto casted = dynamic_cast<const NodeVariable *>(node)) {
        return scope->get_var(casted->name);
    }

    if (auto casted = dynamic_cast<const NodeLambda *>(node)) {
        auto cloned = casted->clone().get();
        auto lambda_ptr = std::shared_ptr<NodeLambda>(const_cast<NodeLambda *>(cloned));

        return std::make_shared<Value>(
            lambda_ptr,
            scope->get_atomic_type("lambda")
        );
    }

    if (auto casted = dynamic_cast<const NodeUnaryOp *>(node)) {
        auto value = eval_expr(casted->value.get(), scope);

        auto key = std::make_tuple(casted->op, nullptr, value->type);
        auto op = scope->get_op(key);

        auto res = op(nullptr, value);

        return res;
    }

    if (auto casted = dynamic_cast<const NodeBinaryOp *>(node)) {
        auto lhs = eval_expr(casted->lhs.get(), scope);
        auto rhs = eval_expr(casted->rhs.get(), scope);

        auto key = std::make_tuple(casted->op, lhs->type, rhs->type);
        auto op = scope->get_op(key);

        auto res = op(lhs, rhs);

        return res;
    }

    if (auto casted = dynamic_cast<const NodeTuple *>(node)) {

    }

    if (auto casted = dynamic_cast<const NodeFunctionCall *>(node)) {
        auto fn_var = scope->get_var(casted->name);
        auto fn_type = fn_var->type;

        std::vector<std::shared_ptr<Type>> arg_types;

        if (auto product = std::dynamic_pointer_cast<ProductType>(fn_type)) {
            arg_types = product->types;
        } else {
            arg_types.push_back(fn_type);
        }

        auto fn = std::get<std::shared_ptr<NodeLambda>>(fn_var->value);
        auto fn_scope = std::make_shared<Scope>(scope);

        for (size_t i = 0; i < fn->args.size(); i++) {
            fn_scope->init_var(fn->args[i], arg_types[i]);
            fn_scope->set_var(fn->args[i], eval_expr(casted->args[i].get(), scope));
        }

        if (std::holds_alternative<std::unique_ptr<NodeBlock>>(fn->body)) {
            auto body = std::get<std::unique_ptr<NodeBlock>>(fn->body).get();
            auto res = eval(body, fn_scope);
            return res->value;
        }
        
        if (std::holds_alternative<std::unique_ptr<NodeExpr>>(fn->body)) {
            auto body = std::get<std::unique_ptr<NodeExpr>>(fn->body).get();
            auto res = eval_expr(body, fn_scope);
            return res;
        }

        // if (auto block_ptr = std::get_if<std::unique_ptr<NodeBlock>>(&fn->body)) {
        //     if (*block_ptr) {
        //         auto res = eval(block_ptr->get(), fn_scope);
        //         return res->value;
        //     }
        // }

        // if (auto block_ptr = std::get_if<std::unique_ptr<NodeExpr>>(&fn->body)) {
        //     if (*block_ptr) {
        //         auto res = eval_expr(block_ptr->get(), fn_scope);
        //         return res;
        //     }
        // }

        throw std::runtime_error("Unexpected function's block or return expression");
    }

    if (auto casted = dynamic_cast<const NodeInt32Literal *>(node)) {
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

    throw std::runtime_error("Unhandled expression node");
}

}