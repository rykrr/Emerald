CC = g++
CXXFLAGS = -g 
CXXFLAGS += -include ext/types.hh -include ext/macros.hh
CXXFLAGS += -lncurses -lpthread -lSDL2
CXXFLAGS += $(addprefix -D, $(FLAGS))

SOURCES = $(shell find src/ -type f -name '*.cc')
OBJECTS = $(SOURCES:src/%.cc=obj/%.o)

SRCDIRS = $(shell find src/ -type d)
OBJDIRS = $(SRCDIRS:src/%=obj/%)

vpath %.tcc src
vpath %.cc src
vpath %.o obj

emerald: $(OBJDIRS) $(OBJECTS)
	$(CC) $(CXXFLAGS) $(OBJECTS) -o $@
	
$(OBJDIRS):
	mkdir -p $@

obj/%.o: %.cc
	$(CC) $(CXXFLAGS) -c -o $@ $<

PHONY: clean
clean:
	rm -rf obj
	rm -f gmon.out
	rm -f perf.data*
