#!/usr/bin/env python3
"""
tools/build_dawg.py
Compile a TSV wordlist into an ultra-compact Directed Acyclic Word Graph (DAWG / Minimal DFA).

Usage:
  python3 tools/build_dawg.py [input_tsv] [output_dawg]
Default:
  data/words.txt -> data/words.dawg
"""

import os
import struct
import sys

def build_dawg(input_path: str, output_path: str):
    with open(input_path, "r", encoding="utf-8") as f:
        words = []
        for line in f:
            line = line.strip()
            if line and not line.startswith("#"):
                words.append(line.split("\t")[0])

    words = sorted(list(set(words)))

    class Node:
        __slots__ = ("edges", "is_final", "id")
        def __init__(self):
            self.edges = {}
            self.is_final = False
            self.id = 0

    root = Node()
    for w in words:
        curr = root
        for ch in w:
            if ch not in curr.edges:
                curr.edges[ch] = Node()
            curr = curr.edges[ch]
        curr.is_final = True

    # Bottom-up minimization (Daciuk's algorithm)
    node_signatures = {}
    def minimize(node):
        for ch, child in list(node.edges.items()):
            node.edges[ch] = minimize(child)
        sig = (node.is_final, tuple(sorted((ch, id(child)) for ch, child in node.edges.items())))
        if sig in node_signatures:
            return node_signatures[sig]
        node_signatures[sig] = node
        return node

    min_root = minimize(root)

    # Breadth-first traversal to assign contiguous state IDs (root = 0)
    states = [min_root]
    min_root.id = 0
    visited = {id(min_root)}

    queue = [min_root]
    while queue:
        curr = queue.pop(0)
        for ch, child in sorted(curr.edges.items()):
            if id(child) not in visited:
                child.id = len(states)
                visited.add(id(child))
                states.append(child)
                queue.append(child)

    # Binary layout:
    # Header: "TBD1" (4B) | num_states (4B uint32) | num_words (4B uint32)
    # Per State:
    #   flags (1B): [is_final (bit 7) | num_edges (bits 0..6)]
    #   edges: [char (2B uint16) | target_state_id (2B uint16)] * num_edges
    buf = bytearray()
    buf.extend(b"TBD1")
    buf.extend(struct.pack("<II", len(states), len(words)))

    for s in states:
        is_final_bit = 0x80 if s.is_final else 0
        num_edges = len(s.edges)
        assert num_edges < 128, f"Too many edges in state {s.id}"
        buf.append(is_final_bit | num_edges)
        for ch, child in sorted(s.edges.items()):
            code = ord(ch)
            assert code <= 0xFFFF, f"Character {ch} exceeds uint16 BMP"
            buf.extend(struct.pack("<HH", code, child.id))

    os.makedirs(os.path.dirname(os.path.abspath(output_path)), exist_ok=True)
    with open(output_path, "wb") as f:
        f.write(buf)

    in_size = os.path.getsize(input_path)
    out_size = len(buf)
    print(f"Compiled {input_path} -> {output_path}")
    print(f"  Words: {len(words)}, States: {len(states)}")
    print(f"  Input size : {in_size:,} bytes ({in_size/1024:.2f} KB)")
    print(f"  Output size: {out_size:,} bytes ({out_size/1024:.2f} KB)")
    print(f"  Compression: {out_size/in_size*100:.1f}% of original size")

if __name__ == "__main__":
    in_file = sys.argv[1] if len(sys.argv) > 1 else "data/words.txt"
    out_file = sys.argv[2] if len(sys.argv) > 2 else "data/words.dawg"
    build_dawg(in_file, out_file)
