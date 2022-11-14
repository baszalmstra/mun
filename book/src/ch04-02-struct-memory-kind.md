## Struct Memory Kind

By default, Mun is a garbage collected language.
This means that memory is _allocated_ on the heap and automatically _freed_ by the Mun Runtime when your memory goes out of scope.
Sometimes this behavior is undesired, and you want to manually control when a value is freed.

Mun allows you to specify this so-called _memory kind_ in a `struct` definition: `gc` for garbage collection or `value` to pass a `struct` by value; defaulting to `gc` when neither is specified.
Listing 4-9 shows the previously created struct definition of a `Vector2`, which has the default `gc` memory kind.

<!-- HACK: Add an extension to support hiding of Mun code -->

```rust,ignore
{{#include ../listings/ch04-structs/listing09.mun}}
```

<span class="caption">Listing 4-9: A record `struct` definition for a 2D vector, defaulting to the `gc` memory kind</span>

To manually specify the memory kind, add round brackets containing either `gc` or `value` after the `struct` keyword, as illustrated in Listing 4-10.

<!-- HACK: Add an extension to support hiding of Mun code -->

```rust,ignore
{{#include ../listings/ch04-structs/listing10.mun}}
```

<span class="caption">Listing 4-10: A record `struct` definition for a 2D vector, with the `value` memory kind</span>
