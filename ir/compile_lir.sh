#!/bin/sh
# This script is meant to be executed from the root of the project.

cargo run -p ir lir ir/test.lir -o ir/target/test.s &&
as ir/target/test.s -o ir/target/test.o &&
gcc -no-pie ir/target/test.o -o ir/target/test

