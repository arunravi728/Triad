# Triad
A toy operating system written in Rust. The name is a homage to [Operating Systems: Three Easy Pieces (OSTEP)](https://pages.cs.wisc.edu/~remzi/OSTEP/), a foundational book on operating systems. The project is structured around the book's three fundamental pillars -

1.  Virtualization
2.  Concurrency
3.  Persistence

## Build
The target triple for Rust is provided in `triad-llvm-target.jsonc`. However, the internal Rust parser only supports strict `.json` files. To generate the target triple `.json` file, run - 

```bash
python3 remove-json-comments.py triad-llvm-target.jsonc
```

After the `.json` file is created, we can build the kernel -

```bash
cargo build
```
