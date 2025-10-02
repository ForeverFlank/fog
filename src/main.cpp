#include <fstream>
#include <iomanip>
#include <iostream>
#include <memory>
#include <stdexcept>
#include <typeinfo>
#include <unordered_map>
#include <vector>
#include <utility>

#include "ast_nodes.h"
#include "ast_parser.h"
#include "lexer.h"
#include "interpreter.h"

const std::map<fog::TokenType, std::string> TOKEN_TYPE_NAMES = {
    { fog::TokenType::TERMINATOR,   "TERMINATOR" },
    { fog::TokenType::ASSIGN,       "ASSIGN"     },
    { fog::TokenType::LBRACE,       "LBRACE"     },
    { fog::TokenType::RBRACE,       "RBRACE"     },
    { fog::TokenType::LPAREN,       "LPAREN"     },
    { fog::TokenType::RPAREN,       "RPAREN"     },
    { fog::TokenType::IDENTIFIER,   "IDENTIFIER" },
    { fog::TokenType::LET,          "LET"        },
    { fog::TokenType::CONST,        "CONST"      },
    { fog::TokenType::INT,          "INT"        },
    { fog::TokenType::FLOAT,        "FLOAT"      },
    { fog::TokenType::STRING,       "STRING"     },
    { fog::TokenType::TRUE,         "TRUE"       },
    { fog::TokenType::FALSE,        "FALSE"      },
    { fog::TokenType::ARROW,        "ARROW"      },
    { fog::TokenType::FATARROW,     "FATARROW"   },
    { fog::TokenType::COLON,        "COLON"      },
    { fog::TokenType::COMMA,        "COMMA"      },
    { fog::TokenType::RETURN,       "RETURN"     },
    { fog::TokenType::IF,           "IF"         },
    { fog::TokenType::ELSE,         "ELSE"       },
    { fog::TokenType::WHILE,        "WHILE"      },
    { fog::TokenType::PLUS,         "PLUS"       },
    { fog::TokenType::MINUS,        "MINUS"      },
    { fog::TokenType::STAR,         "STAR"       },
    { fog::TokenType::SLASH,        "SLASH"      },
    { fog::TokenType::EQ,           "EQ"         },
    { fog::TokenType::NEQ,          "NEQ"        },
    { fog::TokenType::LT,           "LT"         },
    { fog::TokenType::LTE,          "LTE"        },
    { fog::TokenType::GT,           "GT"         },
    { fog::TokenType::GTE,          "GTE"        }
};

void print_tokens(std::vector<fog::Token> &tokens) {
    for (size_t i = 0; i < tokens.size(); i++) {
        std::cout
            << std::setw(4) << i
            << std::setw(12) << TOKEN_TYPE_NAMES.at(tokens[i].type) << " | "
            << tokens[i].value << std::endl;
    }
    std::cout << std::endl;
}

void print_ast(const fog::ASTNode *node, int level = 0) {
    if (!node) return;

    std::string prefix;
    for (int i = 0; i < level; i++) {
        prefix += "  ";
    }
    if (level > 0) {
        prefix[2 * level - 2] = '-';
    }

    if (auto casted = dynamic_cast<const fog::NodeBlock *>(node)) {
        std::cout << prefix << "Block" << std::endl;
        for (auto &child : casted->nodes) {
            print_ast(child.get(), level + 1);
        }
    }

    if (auto casted = dynamic_cast<const fog::NodeDeclare *>(node)) {
        std::cout << prefix << "Declare (";
        std::cout << "is_const: " << casted->is_const << ", ";
        std::cout << "var_name: " << casted->var_name << ")" << std::endl;
        print_ast(casted->type.get(), level + 1);
        if (casted->value != nullptr) {
            print_ast(casted->value.get(), level + 1);
        }
    }

    if (auto casted = dynamic_cast<const fog::NodeAssign *>(node)) {
        std::cout << prefix << "Assign (";
        std::cout << "var_name: " << casted->var_name << ")" << std::endl;
        print_ast(casted->value.get(), level + 1);
    }

    if (auto casted = dynamic_cast<const fog::NodeReturn *>(node)) {
        std::cout << prefix << "Return" << std::endl;
        print_ast(casted->value.get(), level + 1);
    }

    if (auto casted = dynamic_cast<const fog::NodeVariable *>(node)) {
        std::cout << prefix << "Variable (";
        std::cout << "name: " << casted->name << ")" << std::endl;
    }

    if (auto casted = dynamic_cast<const fog::NodeLambda *>(node)) {
        std::cout << prefix << "Lambda (";
        std::cout << "args: [";
        bool b = false;
        for (auto &arg : casted->args) {
            if (b) std::cout << ", ";
            std::cout << arg;
            b = true;
        }
        std::cout << "])" << std::endl;
        if (casted->body.index() == 0) {
            print_ast(std::get<0>(casted->body).get(), level + 1);
        } else if (casted->body.index() == 1) {
            print_ast(std::get<1>(casted->body).get(), level + 1);
        }
    }

    if (auto casted = dynamic_cast<const fog::NodeUnaryOp *>(node)) {
        std::cout << prefix << "UnaryOp (";
        std::cout << "op: " << casted->op << ")" << std::endl;
        print_ast(casted->value.get(), level + 1);
    }
    
    if (auto casted = dynamic_cast<const fog::NodeBinaryOp *>(node)) {
        std::cout << prefix << "BinaryOp (";
        std::cout << "op: " << casted->op << ")" << std::endl;
        print_ast(casted->lhs.get(), level + 1);
        print_ast(casted->rhs.get(), level + 1);
    }

    if (auto casted = dynamic_cast<const fog::NodeFunctionCall *>(node)) {
        std::cout << prefix << "FunctionCall (";
        std::cout << "name: " << casted->name << ")" << std::endl;
        for (auto &child : casted->args) {
            print_ast(child.get(), level + 1);
        }
    }

    if (auto casted = dynamic_cast<const fog::NodeInt32Literal *>(node)) {
        std::cout << prefix << "Int32Literal (";
        std::cout << "value: " << casted->value << ")" << std::endl;
    }

    if (auto casted = dynamic_cast<const fog::NodeAtomicType *>(node)) {
        std::cout << prefix << "AtomicType (";
        std::cout << "name: " << casted->name << ")" << std::endl;
    }

    if (auto casted = dynamic_cast<const fog::NodeProductType *>(node)) {
        std::cout << prefix << "TupleType" << std::endl;
        for (auto &child : casted->types) {
            print_ast(child.get(), level + 1);
        }
    }

    if (auto casted = dynamic_cast<const fog::NodeMapType *>(node)) {
        std::cout << prefix << "MapType" << std::endl;
        print_ast(casted->domain.get(), level + 1);
        print_ast(casted->codomain.get(), level + 1);
    }
}

int main(int argc, char *argv[]) {
    if (argc < 2) {
        std::cerr << "Usage: " << argv[0] << " <file-path>\n";
        return 1;
    }

    const char *path = argv[1];

    std::ifstream file(path);
    if (!file) {
        std::cerr << "Failed to open file: " << path << "\n";
        return 1;
    }

    std::string source{
        std::istreambuf_iterator<char>(file),
        std::istreambuf_iterator<char>()
    };

    file.close();

    fog::Lexer lexer(source);
    std::vector<fog::Token> tokens = lexer.tokenize();

    // print_tokens(tokens);

    fog::ASTParser ast_parser(tokens);
    std::unique_ptr<fog::NodeBlock> main_block = ast_parser.parse_main();

    // print_ast(main_block.get());

    fog::Interpreter interpreter;
    interpreter.eval(main_block.get());

    std::cout << std::endl;
    for (auto &item : interpreter.global_scope->variables) {
        std::string val;
        try {
            val = std::to_string(std::get<int32_t>(item.second->value));
        } catch (std::exception e) {
            val = "<?>";
        }
        std::cout << item.first << " = " << val << std::endl;
    }

    return 0;
}