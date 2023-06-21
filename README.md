# Movable Tree CRDT

A movable and immutable tree data structure. It can be used to model move
operation in tree, while persisting the history version of tree with low cost.
The time complexity of clone a tree is O(1), which would reuse the memory.

It serves as a proof of concept that preserving all history versions of
modifying a tree's hierarchy can be efficient.

It doesn't guarantee the order of siblings in the current version.

## Example

The following example create a tree with

```log
  1
 / \
2   3
```

```no_run
use movable_tree::Forest;
let mut forest: Forest<usize> = Forest::new();
forest.mov(1, None);
forest.mov(2, Some(1));
forest.mov(3, Some(1));
```

To move 2 under 3

```log
1
|
3
|
2
```

```no_run
# use movable_tree::Forest;
# let mut forest: Forest<usize> = Forest::new();
forest.mov(2, Some(3));
```

Move op that would cause cycle in tree is forbidden and return Err.

# Performance

## CRDT

> In this benchmark, we assume there are 10K nodes and the depth of tree is
> within 4

By using log-spaced snapshots to store the history, the duration of applying 1M
move ops for tree crdt is

_Tested on M1_

| n    | Total time |
| :--- | :--------- |
| 10K  | 18 ms      |
| 100K | 205 ms     |
| 1M   | 2.07 s     |

By using undoable tree crdt, the duration of applying 1M move ops is

| n    | Total time |
| :--- | :--------- |
| 10K  | 0.9 ms     |
| 100K | 12 ms      |
| 1M   | 122 ms     |

## Preserve History by Immutable Data Structure

The cost of recording the history of inserting n nodes. The history contains the
versions with 1 node, 2 nodes, ..., n nodes.

Tested on M1.

| n    | Memory Usage | Total time | Dropping time | Applying time |
| :--- | :----------- | :--------- | :------------ | :------------ |
| 10K  | 3 MB         | 19 ms      | 9 ms          | 1 ms          |
| 100K | 39 MB        | 337 ms     | 197 ms        | 30 ms         |
| 1M   | 450 MB       | 7.6 s      | 3 s           | 426 ms        |
