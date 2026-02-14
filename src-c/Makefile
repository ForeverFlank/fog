CC = gcc
CFLAGS = -Wall -Wextra -std=c11 -g

TARGET = fog

OBJS = main.o types.o lexer.o # ast_parser.o interpreter.o

$(TARGET): $(OBJS)
	$(CC) $(CFLAGS) -o $(TARGET) $(OBJS)

main.o: main.c lexer.h types.h
	$(CC) $(CFLAGS) -c main.c

types.o: types.c types.h
	$(CC) $(CFLAGS) -c types.c

lexer.o: lexer.c lexer.h types.h
	$(CC) $(CFLAGS) -c lexer.c

# ast_parser.o: ast_parser.c ast_parser.h ast_nodes.h
#	$(CC) $(CFLAGS) -c ast_parser.c

# interpreter.o: interpreter.c interpreter.h ast_nodes.h
#	$(CC) $(CFLAGS) -c interpreter.c

clean:
	rm -f $(OBJS) $(TARGET)
