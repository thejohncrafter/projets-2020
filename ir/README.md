
# Intermediate Representations

## HIR

> High Intermediate Representation

A language where :
 * All values are `Data`'s : only contains a type identifier and a value
   (may be simply represented by two words)

   Structures are not explicit, we only have pointers
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

This language is _not_ typed : everything is `Data`.

This means it is not memory safe !

This language is used to deconstruct Petit Julia's types and to make
explicit all type casts.

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

Only two abstractions over assembly remain here : functions
and pseudo-registers.

For a first implementation, we may want to make everything live
on the stack.

# Compilation

> Petit Julia --> HIR --> LIR --> Assembly

The first goal is to get this pipeline working.

The two intermediate languages are designed to leave some space
for optimizations, like SSA and efficient register allocation.

