#!/bin/sh

mkdir -p compil_bundle
cp -r ../automata compil_bundle/automata
cp -r ../parsergen compil_bundle/parsergen
cp -r ../parser compil_bundle/parser
cp -r ../contrib compil_bundle/contrib
cp compil_cargo.txt compil_bundle/Cargo.toml
cp ../Makefile compil_bundle/Makefile
