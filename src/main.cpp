#include <fstream>
#include <iomanip>
#include <iostream>
#include <stdexcept>
#include <typeinfo>
#include <unordered_map>
#include <vector>

#include "ast_nodes.h"
#include "ast_parser.h"
#include "lexer.h"

const std::map<fog::TokenType, std::string> TOKEN_TYPE_NAMES = {
    {fog::TokenType::TERMINATOR, "TERMINATOR"},
    {fog::TokenType::ASSIGN, "ASSIGN"},
    {fog::TokenType::LBRACE, "LBRACE"},
    {fog::TokenType::RBRACE, "RBRACE"},
    {fog::TokenType::LPAREN, "LPAREN"},
    {fog::TokenType::RPAREN, "RPAREN"},
    {fog::TokenType::IDENTIFIER, "IDENTIFIER"},
    {fog::TokenType::LET, "LET"},
    {fog::TokenType::CONST, "CONST"},
    {fog::TokenType::INT, "INT"},
    {fog::TokenType::FLOAT, "FLOAT"},
    {fog::TokenType::STRING, "STRING"},
    {fog::TokenType::TRUE, "TRUE"},
    {fog::TokenType::FALSE, "FALSE"},
    {fog::TokenType::ARROW, "ARROW"},
    {fog::TokenType::COLON, "COLON"},
    {fog::TokenType::COMMA, "COMMA"},
    {fog::TokenType::RETURN, "RETURN"},
    {fog::TokenType::IF, "IF"},
    {fog::TokenType::ELSE, "ELSE"},
    {fog::TokenType::WHILE, "WHILE"},
    {fog::TokenType::PLUS, "PLUS"},
    {fog::TokenType::MINUS, "MINUS"},
    {fog::TokenType::STAR, "STAR"},
    {fog::TokenType::SLASH, "SLASH"},
    {fog::TokenType::EQ, "EQ"},
    {fog::TokenType::NEQ, "NEQ"},
    {fog::TokenType::LT, "LT"},
    {fog::TokenType::LTE, "LTE"},
    {fog::TokenType::GT, "GT"},
    {fog::TokenType::GTE, "GTE"}
};

void print_tokens(std::vector<fog::Token> &tokens) {
    for (size_t i = 0; i < tokens.size(); i++) {
        std::cout
            << std::setw(4) << i
            << std::setw(12) << TOKEN_TYPE_NAMES.at(tokens[i].type) << " | "
            << tokens[i].value << std::endl;
    }
}

void print_ast(fog::ASTNode *node) {
    if (typeid(*node) == typeid(fog::NodeBlock)) {
        std::cout << "NodeBlock" << std::endl;
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

    print_tokens(tokens);

    fog::ASTParser ast_parser(tokens);
    std::unique_ptr<fog::NodeBlock> main_block = ast_parser.parse_main();

    print_ast(main_block.get());

    return 0;
}