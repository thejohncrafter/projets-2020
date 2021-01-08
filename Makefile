COLOR ?= auto # Valid COLOR options: {always, auto, never}
CARGO = cargo --color $(COLOR)

.PHONY: all bench build check clean doc install publish run test update

all: build

bench:
	@$(CARGO) bench

build-pjuliac:
	@$(CARGO) build --release --bin parser
	ln -sf ./target/release/parser ./pjuliac

run-pjuliac: build-pjuliac
	@$(CARGO) run --bin parser

test-pjuliac: build-pjuliac
	cd contrib/compil; ./test.sh -1 ../../target/release/parser && ./test.sh -2 ../../target/release/parser

tarball-pjuliac:
	mkdir -p /tmp/Lahfa-Marquet
	cp -r automata /tmp/Lahfa-Marquet/
	cp -r parsergen /tmp/Lahfa-Marquet/
	cp -r parser /tmp/Lahfa-Marquet/
	cp -r contrib /tmp/Lahfa-Marquet
	cp Makefile /tmp/Lahfa-Marquet/
	cp compil/compil_cargo.txt /tmp/Lahfa-Marquet/Cargo.toml
	cp rapports/compil/rapport_miprojet.pdf /tmp/Lahfa-Marquet/rapport.pdf
	cd /tmp; zip -r -9 Lahfa-Marquet.zip ./Lahfa-Marquet/**

check:
	@$(CARGO) check

clean:
	@$(CARGO) clean

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
