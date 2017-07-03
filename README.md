# `jagged_array` - A cache-friendly `Vec<Box<[T]>>` replacement

`Vec<Vec<_>>` is a really commonly-used idiom, but is absolutely awful for cache
locality. You can store a flat vector and dimensions if you have a 2D,
matrix-esque array, but for jagged data you need something more specialized.

This is only for when the inner vectors are immutable though. You can mutate the
elements, but changing the lengths of the inner vectors requires creating a new
`JaggedArray`.
