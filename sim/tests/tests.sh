#!/bin/sh

SIM=../../target/release/sim

if [ -f "$SIM" ]; then
	echo "Testing binary operators (and some bus operations"
	echo "by the way) :"
	$SIM test bin_ops
	echo
	echo "Testing registers :"
	$SIM test reg
	echo
	echo "Testing RAM :"
	$SIM test ram
else
	echo "The simulator must be build before testing !"
	echo "See README.md."
	echo ""
	echo "You might also see this message because you"
	echo "are not running this script from sim/tests/."
fi

