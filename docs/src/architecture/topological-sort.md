# Topological Sorting

How block execution order is determined.

## The Problem

DSP blocks must execute in the correct order:

```
Oscillator -> Gain -> Output
```

The oscillator must run first (it produces audio), then gain (processes it), then output (collects it).

## Kahn's Algorithm

bbx_audio uses Kahn's algorithm for topological sorting:

```rust
fn topological_sort(&self) -> Vec<BlockId> {
    let mut in_degree = vec![0; self.blocks.len()];
    let mut adjacency_list: HashMap<BlockId, Vec<BlockId>> = HashMap::new();

    // Build graph
    for connection in &self.connections {
        adjacency_list.entry(connection.from).or_default().push(connection.to);
        in_degree[connection.to.0] += 1;
    }

    // Find blocks with no dependencies
    let mut queue = Vec::new();
    for (i, &degree) in in_degree.iter().enumerate() {
        if degree == 0 {
            queue.push(BlockId(i));
        }
    }

    // Process in dependency order
    let mut result = Vec::new();
    while let Some(block) = queue.pop() {
        result.push(block);
        if let Some(neighbors) = adjacency_list.get(&block) {
            for &neighbor in neighbors {
                in_degree[neighbor.0] -= 1;
                if in_degree[neighbor.0] == 0 {
                    queue.push(neighbor);
                }
            }
        }
    }

    result
}
```

## Algorithm Steps

1. **Calculate in-degrees**: Count incoming connections for each block
2. **Initialize queue**: Add blocks with no inputs (sources)
3. **Process queue**:
   - Remove a block from queue
   - Add to result
   - Decrement in-degree of connected blocks
   - Add newly zero-degree blocks to queue
4. **Result**: Blocks in valid execution order

## Example

Given this graph:

```
Osc (0) -> Gain (1) -> Output (2)
           ^
LFO (3) --/
```

Connections:
- 0 -> 1
- 3 -> 1
- 1 -> 2

In-degrees:
- Block 0: 0 (no inputs)
- Block 1: 2 (from 0 and 3)
- Block 2: 1 (from 1)
- Block 3: 0 (no inputs)

Processing:
1. Queue: [0, 3] (in-degree 0)
2. Pop 0, result: [0], decrement block 1
3. Pop 3, result: [0, 3], decrement block 1 (now 0)
4. Queue: [1], pop 1, result: [0, 3, 1], decrement block 2 (now 0)
5. Queue: [2], pop 2, result: [0, 3, 1, 2]

## Cycle Detection

If the result length doesn't match block count, there's a cycle:

```rust
if result.len() != self.blocks.len() {
    // Graph has a cycle - invalid!
}
```

Cycles are not allowed in DSP graphs (would cause infinite loops).
