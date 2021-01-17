
# Intermediate Representations

## HIR

> High Intermediate Representation

A language where :
 * All values are `Data`'s : only contains a type identifier and a value
   (may be simply represented by two words)
 * Structures are declared, so that `Access` instructions can find a
   specific value from a structure
 * No nested expressions : code is composed of blocs, control structures
   and declarations

   This means that expressions like `2 * (x + 1)` must be "flattened".
   We may introduce new temprary variables in the process.
 * Every computation is a `Call` :
    * Arithmetics
    * Also memory access for structures (e.g. call something like `Access`
      and give it a pointer and an offset)
    * One specific callable primitive : `Cast`, which ensures some `Data`
      has the right type
    * Calls may not map 1:1 to instructions
    * Function calls are done with some `Call` primitive, by giving
      a function identifier (which needs not be a `Data` object, as
      we do not need lambdas)
 * All functions have a return value, even if it is `Nothing`.

This language is _not_ typed : everything is `Data`.

This language is used to deconstruct Petit Julia's nested expressions
and to make explicit all type casts.

## LIR

> Low Intermediate Representation

A language where
 * Instructions map trivially to assembly (hopefully 1:1)
   except function calls
 * Register allocation isn't explicit : everything is stored in
   pseudo-registers (that will be mapped to real registers or to
   stack locations during register allocation).
 * No control structures : flow control is done using labels
   and jumps
 * There remain functions and function calls
   
   Function calls are the only things that do not trivially map
   to assembly : the way arguments are passed and registers are
   saved is handled during the last compilation phase
   (LIR to assembly)
 * Function either return two values or no value
   (we want to return two values as in Petit Julia everything
   is represented by two 64 bit words).

Only two abstractions over assembly remain here : functions
and pseudo-registers.

The compiled functions respect the System V AMD64 ABI.

In this implementation, everything is saved on the stack
(this was the fastest way to get a working version).

# Compilation

> Petit Julia --> HIR --> LIR --> Assembly

The first goal is to get this pipeline working.

The two intermediate languages are designed to leave some space
for optimizations, like SSA and efficient register allocation.

The script `compile_lir.sh` runs compiles `ir/test.lir` to `ir/target/test.s`,
then performs assembly and linking within `ir/target/`.

# Runtime

Both the HIR and the LIR provide ways to call "native functions", which
are defined in `runtime.c`.

A native call to a function `foo` in any of the IRs will be compiled to
a call to `native_foo`. `native_foo` should then be declared in `runtime.c`.

Function must return two values (the type of the return value, and the actual
return value). To interoperate with C, the first two arguments of every
native function are :
 * a pointer to `ret_val_ty`
 * a pointer to `ret_val_val`

Every argument of the native function is 64 bits long
(this is required by the representation of values that we chose).

