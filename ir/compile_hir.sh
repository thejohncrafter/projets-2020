#!/bin/sh
# This script is meant to be executed from the root of the project.

cargo run -p ir hir ir/test.hir -o ir/target/test.s &&
as ir/target/test.s -o ir/target/test.o &&
gcc -c ir/runtime.c -o ir/target/runtime.o &&
gcc -no-pie ir/target/test.o ir/target/runtime.o -o ir/target/test

