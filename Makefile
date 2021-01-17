COLOR ?= auto # Valid COLOR options: {always, auto, never}
CARGO = cargo --color $(COLOR)

.PHONY: all bench build check clean doc install publish run test update

COMPILER = $(shell pwd)/target/release/compiler

all: build

bench:
	@$(CARGO) bench

build-pjuliac:
	@$(CARGO) build --release --bin compiler
	ln -sf ./target/release/compiler ./pjuliac

build-runtime:
	@$(CC) -c ir/runtime.c -o ./target/release/runtime.o

make-available-runtime: build-runtime
	ln -sf $(shell pwd)/target/release/runtime.o contrib/compil/runtime.o

run-pjuliac: build-pjuliac build-runtime
	@$(CARGO) run --bin compiler

test-pjuliac: build-pjuliac make-available-runtime
	cd contrib/compil; ./test.sh -all ../../target/release/compiler

test-pjuliac-verbose: build-pjuliac make-available-runtime
	cd contrib/compil; ./test.sh -v1 $(COMPILER) && ./test.sh -v2 $(COMPILER) && ./test.sh -v3 $(COMPILER)

tarball-pjuliac:
	mkdir -p /tmp/Lahfa-Marquet
	cp -r automata /tmp/Lahfa-Marquet/
	cp -r parsergen /tmp/Lahfa-Marquet/
	cp -r parser /tmp/Lahfa-Marquet/
	cp -r contrib /tmp/Lahfa-Marquet
	cp -r ir /tmp/Lahfa-Marquet
	cp -r compiler /tmp/Lahfa-Marquet
	cp Makefile /tmp/Lahfa-Marquet/
	cp Cargo.toml /tmp/Lahfa-Marquet/Cargo.toml
	rm -r /tmp/Lahfa-Marquet/**/target
	sed -i 's/"sim", //' /tmp/Lahfa-Marquet/Cargo.toml
	cp rapports/compil/rapport_miprojet.pdf /tmp/Lahfa-Marquet/rapport_p1.pdf
	cp rapports/compil/rapport_complet.pdf /tmp/Lahfa-Marquet/rapport_p2.pdf
	cd /tmp; zip -r -9 Lahfa-Marquet.zip ./Lahfa-Marquet/**
	mv /tmp/Lahfa-Marquet.zip .

check:
	@$(CARGO) check

clean:
	@$(CARGO) clean
	rm -rf contrib/compil/**/*.o
	rm -rf contrib/compil/**/*.s
	rm -rf contrib/compil/**/*.hir
	rm -rf contrib/compil/**/*.lir

doc:
	@$(CARGO) doc

install: build
	@$(CARGO) install

publish:
	@$(CARGO) publish

run: build
	@$(CARGO) run

test: build
	@$(CARGO) test

update:
	@$(CARGO) update
