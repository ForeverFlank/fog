CXX = g++
CXXFLAGS = -Wall -Wextra -std=c++23

TARGET = fog

OBJS = main.o lexer.o

$(TARGET): $(OBJS)
	$(CXX) $(CXXFLAGS) -o $(TARGET) $(OBJS)

main.o: main.cpp lexer.h
	$(CXX) $(CXXFLAGS) -c main.cpp

lexer.o: lexer.cpp lexer.h
	$(CXX) $(CXXFLAGS) -c lexer.cpp

clean:
	rm -f $(OBJS)
