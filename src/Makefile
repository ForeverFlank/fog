CXX = g++
CXXFLAGS = -Wall -Wextra -std=c++23

TARGET = fog

OBJS = main.o lexer.o ast_parser.o

$(TARGET): $(OBJS)
	$(CXX) $(CXXFLAGS) -o $(TARGET) $(OBJS)

main.o: main.cpp lexer.h ast_parser.h ast_nodes.h
	$(CXX) $(CXXFLAGS) -c main.cpp

lexer.o: lexer.cpp lexer.h
	$(CXX) $(CXXFLAGS) -c lexer.cpp

ast_parser.o: ast_parser.cpp ast_parser.h ast_nodes.h
	$(CXX) $(CXXFLAGS) -c ast_parser.cpp

clean:
	rm -f $(OBJS)
