# POC of Movable Tree CRDT

The cost of recording the history of inserting n nodes. The history contains the
versions with 1 node, 2 nodes, ..., n nodes.

Tested on M1.

| n    | Memory Usage | Total time | Dropping time | Applying time |
| :--- | :----------- | :--------- | :------------ | :------------ |
| 10K  | 3 MB         | 19 ms      | 9 ms          | 1 ms          |
| 100K | 39 MB        | 337 ms     | 197 ms        | 30 ms         |
| 1M   | 450 MB       | 7.6 s      | 3 s           | 426 ms        |
