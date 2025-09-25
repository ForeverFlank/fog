#include <fstream>
#include <iomanip>
#include <iostream>
#include <stdexcept>
#include <unordered_map>
#include <vector>

#include "lexer.h"
#include "ast_nodes.h"
#include "ast_parser.h"

int main(int argc, char* argv[]) {
    if (argc < 2) {
        std::cerr << "Usage: " << argv[0] << " <file-path>\n";
        return 1;
    }

    const char* path = argv[1];
    
    std::ifstream file(path);
    if (!file) {
        std::cerr << "Failed to open file: " << path << "\n";
        return 1;
    }

    std::string source{std::istreambuf_iterator<char>(file),
                       std::istreambuf_iterator<char>()};

    file.close();

    fog::Lexer lexer(source);
    std::vector<fog::Token> tokens = lexer.Tokenize();
    // for (auto tkn : tokens) {
    //     std::cout << std::setw(12) << fog::TOKEN_TYPE_NAMES.at(tkn.type)
    //               << " | " << tkn.value << std::endl;
    // }
    
    fog::ASTParser ast_parser(tokens);
    auto block = ast_parser.ParseMain();
    // std::cout << (block == nullptr) << std::endl;
    // for (auto &node : block->nodes) {
    //     std::cout << node.get() << std::endl;
    // }

    return 0;
}