# Low-Level Systems in Rust

A guide to a CS:APP-style systems course taught entirely in Rust. It has fourteen modules and five capstone projects, and takes about 10–12 weeks at 6–8 hours a week.

This README summarises the full course. Each module section below covers the main ideas, what you build or break, and the one-line rule the module ends on.

> **Thesis.** C teaches low-level concepts by letting you break things. Rust teaches the same concepts by making you *name* them. Every C bug class has a Rust type, lint or `unsafe` contract that exists because of it. Each module therefore does three things:
> 1. it shows the machine,
> 2. it shows the C habit,
> 3. it shows which Rust construct encodes the rule.

---

## Contents

- [Who it's for](#who-its-for)
- [How to work through it](#how-to-work-through-it)
- [Setup](#setup)
- [Module map](#module-map)
- [Modules 0–13](#module-0--the-toolkit-seeing-what-the-machine-sees)
- [Capstone projects](#capstone-projects)
- [Suggested pacing](#suggested-pacing)
- [Reading list](#reading-list)
- [The rules, in one place](#the-rules-in-one-place)

---

## Who it's for

You should already be comfortable with ownership, `Drop`, `Rc<RefCell<_>>` and `impl` vs `dyn`. Some exposure to byte-level layouts also helps, for example Solana's Token-2022 accounts (the 82-byte mint base, `COption`'s 4-byte tag and TLV extension entries).

The course explains *why* those things look the way they do. Throughout the material, **↔ you've seen this** marks a link back to earlier Rust or Solana work, such as Anchor, the SPL token programs and Token-2022.

## How to work through it

Each module follows the same four-step loop:

1. **Predict.** Before you run any snippet, write down what you expect: a size, an address pattern or an output. The wrong predictions are where the learning happens.
2. **Run and look.** Print the value, disassemble the code or run it under Miri. Don't trust a mental model you haven't checked against the machine.
3. **Break it on purpose.** Every module has one exercise where you cause the bug that C would allow, then watch Rust or Miri catch it.
4. **Write the rule.** Each module ends with a one-line rule. Rewrite it in your own words.

The exercises have **no published solutions**. If you get stuck, take your attempt and your prediction to a peer or mentor rather than starting from a blank page.

## Setup

- A current **stable** Rust toolchain on edition 2024.
- **Nightly**, for Miri and the sanitizers: `rustup +nightly component add miri`.
- **Linux x86-64**, or WSL2. macOS works for everything except the raw syscalls in Module 12.

---

## Module map

| # | Module | Core question |
|---|--------|---------------|
| 0 | The toolkit | How do I see what the compiler actually produced? |
| 0.5 | Reading x86-64 assembly | What do the ~30 instructions I'll meet mean? |
| 1 | Bits, bytes and integers | What does a bit pattern *mean*? |
| 2 | The process memory map | Where does each value live, and for how long? |
| 3 | Pointers, references and `unsafe` | What does a pointer promise, and who checks it? |
| 4 | Arrays, slices and strings | Why do buffer overflows exist, and how does a fat pointer prevent them? |
| 5 | Layout: size, alignment, padding, `repr` | How are types laid out in memory? |
| 6 | Dynamic memory | How do `Vec` and `malloc` actually work? |
| 7 | Functions, ABI and dispatch | Which code runs, and where does its data come from? |
| 8 | FFI | How do Rust and C talk safely? |
| 9 | Bytes on the wire | How do I parse untrusted binary data? |
| 10 | Caches and performance | Why is memory layout performance? |
| 11 | Concurrency, atomics, ordering | How do data races become type errors? |
| 12 | The OS boundary | What do syscalls, file descriptors and virtual memory look like from Rust? |
| 13 | `no_std` and bare metal | What's left when the OS is gone? |

---

## Module 0 — The toolkit: seeing what the machine sees

You can't learn low-level programming by reading source. You learn it by looking at what the source became. Set these tools up before anything else:

| Tool | What it shows | Invoke |
|------|---------------|--------|
| Compiler Explorer (godbolt.org) | Rust and its assembly side by side | Compare `-C opt-level=0` and `-C opt-level=3` |
| `cargo-show-asm` | Assembly or LLVM IR for one function | `cargo asm --lib my_crate::my_fn` |
| **Miri** | Undefined behaviour: out-of-bounds access, use-after-free, misaligned or uninitialised reads, aliasing violations, data races | `cargo +nightly miri run` / `miri test` |
| AddressSanitizer | The same bug classes at native speed, including in linked C code | `RUSTFLAGS="-Zsanitizer=address" cargo +nightly run --target x86_64-unknown-linux-gnu` |
| gdb / lldb | Stepping, registers and memory dumps | `rust-gdb target/debug/app` |

Two more tools come up often: `std::hint::black_box`, which stops the optimiser deleting code you're observing, and `objdump -d` / `nm`.

**The C habit to drop:** "It ran and printed the right thing" is weak evidence, because UB often appears to work. Safe Rust can't have UB, but `unsafe` code can. From Module 3 on, **every `unsafe` exercise runs under Miri** before you trust it.

**Exercises:**
- Compare the assembly for `a + b` at `opt-level=0` and at `opt-level=3`, and find the overflow check.
- Watch a summing loop disappear at `opt-level=3`.
- Get your first Miri error by reading past the end of an array.

> **Rule:** Source is a request; assembly is what happened. Check the machine before you trust the model.

## Module 0.5 — Reading x86-64 assembly

This is a reference page. You need to *read* about 30 instructions, not write them.

- **Syntax:** Intel syntax, `op dest, src`, with the result on the left. Lines starting with `.` are assembler directives; skip them.
- **Registers:** there are 16 general-purpose 64-bit registers (`rax`, `rdi`, …), and each has 32-, 16- and 8-bit views. Writing a 32-bit register zeroes the upper 32 bits of the full register.
- **Linux calling convention:** arguments go in `rdi, rsi, rdx, rcx, r8, r9`. The return value comes back in `rax`. `rbx, rbp, r12–r15` are callee-saved.
- **Flags:** `ZF`, `CF`, `SF` and `OF` are set by arithmetic and `cmp`/`test`, and read by conditional jumps. `b`/`a` jumps are unsigned comparisons; `l`/`g` jumps are signed.
- **Instructions you'll meet:** `mov`, `movzx`/`movsx`, `lea`, `add`/`sub`, `imul`/`mul`, `div`, bitwise ops, shifts, `cmp`/`test`, `jcc`, `cmovcc`, `setcc`, `call`/`ret`, `push`/`pop`, `ud2`.

**To read a listing:**
1. Skip the `.` directives.
2. Find the arguments in `rdi` and `rsi`.
3. Find every `ret`; the value in `rax` just before it is the return value.
4. A backward jump is a loop and a forward jump is an `if`.
5. Trace a small input, such as `n = 4`, by hand.

**References:**
- Felix Cloutier's x86 reference
- Godbolt's inline instruction docs
- Ed Jorgensen, *x86-64 Assembly Language Programming with Ubuntu*
- CS:APP chapter 3
- Agner Fog's instruction tables

> **Rule:** Result on the left, arguments in `rdi, rsi`, answer in `rax`. Find the jumps, then trace a small input by hand.

## Module 1 — Bits, bytes and integers

A number in memory is a fixed-width bit pattern, and its type decides what the pattern means. C blurs this with implicit conversions and undefined signed overflow. Rust makes every width, conversion and overflow policy explicit.

- **Fixed widths:** `i8`…`i128`, `u8`…`u128`, and `isize`/`usize`, which are the size of a pointer. Rust has no "at least 16 bits" type like C's `int`.
- **Two's complement:** `-x` is stored as `!x + 1`, so one adder circuit handles both signed and unsigned addition.
- **Overflow is a policy you choose:** `wrapping_*`, `checked_*`, `saturating_*` or `overflowing_*`. In C, signed overflow is UB. ↔ This is why Anchor sets `overflow-checks = true` and why SPL code uses `checked_add(...).ok_or(...)`.
- **`as` is sharp:** it never fails, so it truncates, sign-extends or reinterprets. For example, `300_i32 as u8 == 44`, and float-to-int conversion saturates. Prefer `try_from` when a value might not fit.
- **Shifts and bit tricks:** `>>` is arithmetic on signed types and logical on unsigned types. `count_ones`, `leading_zeros` and `rotate_left` are built-in methods.
- **Endianness:** x86-64, ARM64 and Solana's SBF are little-endian. ↔ Borsh and SPL accounts use `u64::from_le_bytes`.
- **Floats (IEEE 754):** `0.1 + 0.2 != 0.3` and `NaN != NaN`. Money never goes in floats, which is why token amounts are integers with a `decimals` field.

**Exercises:**
- Compare shifting and dividing negative numbers.
- Hand-write a popcount using `x & (x - 1)`.
- Label the bit fields of an `f32`.
- Find an addition that panics in debug builds but not in release builds.

> **Rule:** A type is an interpretation of bits. Make every width, conversion and overflow decision on purpose.

## Module 2 — The process memory map

A program's memory has four regions: **text** (code), **static** (literals and globals), **heap** and **stack**. The region a value lives in decides its lifetime.

| Region | Holds | Rust spelling |
|--------|-------|---------------|
| Text | machine code | functions, `fn` pointers |
| Static | literals, globals | `static`, `&'static str` |
| Heap | runtime-sized or long-lived data | `Box`, `Vec`, `String`, `Rc` |
| Stack | locals, arguments, return addresses | every `let` binding |

- **The stack is a bump pointer.** A call subtracts from `rsp` to make room for its frame, and returning adds it back. `main` gets about 8 MiB of stack and spawned threads get 2 MiB.
- **C lets you return `&local`.** Rust refuses with error E0515. The lifetime system is stack discipline, checked by the compiler.
- **The heap:** a `Box` is 8 bytes on the stack pointing at the heap, and dropping it calls `free`. Ownership answers C's question "who calls free?"
- **Statics:** `static mut` references are denied by default in edition 2024. Use atomics or a `Mutex` in a plain `static` instead.

**Exercises:**
- Print addresses across several runs to see ASLR at work.
- Measure stack frame sizes through recursion.
- Trigger a stack overflow and learn about guard pages.
- Find out why `Box::new([0u8; 16 MiB])` can crash in a debug build.

> **Rule:** A value's region decides its lifetime. Lifetimes in Rust are that fact, written in the type.

## Module 3 — Pointers, references and `unsafe`

A C pointer is an address plus a promise the programmer keeps in their head. Rust has two kinds of pointer:
- A **reference** (`&T`, `&mut T`) is a pointer whose promise the compiler checks. It is non-null, aligned, points at a valid `T` and outlives its use. You can have many `&T` or one `&mut T`, never both. That last rule is C's `restrict`, applied everywhere.
- A **raw pointer** (`*const T`, `*mut T`) is a C pointer. Creating one is safe; dereferencing one is `unsafe`.

Key points:
- **`p.add(n)`** moves the pointer by `n * size_of::<T>()` bytes. Computing a pointer more than one past the end of its allocation is already UB.
- **`unsafe` unlocks exactly five things:**
  - dereferencing a raw pointer,
  - calling an `unsafe fn`,
  - touching a `static mut`,
  - implementing an `unsafe trait`,
  - reading a `union` field.

  Everything else is still checked.
- **The contract convention:** an `unsafe fn` lists its preconditions under `# Safety`, and every `unsafe {}` block carries a `// SAFETY:` comment proving them. In edition 2024, the body of an `unsafe fn` is not implicitly unsafe.
- **Null:** safe code spells a nullable pointer `Option<&T>` or `Option<NonNull<T>>`, and it costs zero extra bytes.
- **Provenance:** a pointer carries a permission as well as an address. `addr()`, `map_addr()` and `&raw const`/`&raw mut` work with it directly.
- **Aliasing** is where `unsafe` code really breaks. ↔ This is why Solana's `AccountInfo` data sits behind `Rc<RefCell<&mut [u8]>>`.

**Exercises:**
- Write `my_swap` with `ptr::read` and `ptr::write`.
- Walk an array with raw pointers.
- Find an aliasing bug with Miri.
- Build a tagged pointer with `map_addr`.

> **Rule:** A reference is a pointer plus a proof. `unsafe` means you're writing the proof yourself, so write it down.

## Module 4 — Arrays, slices and strings

In C, arrays decay to bare pointers and forget their length, and buffer overflows follow from that one fact. Rust's fix is the **fat pointer**: a pointer that carries its length with it.

| Type | Elements live | Handle size | Fields |
|------|---------------|-------------|--------|
| `[T; N]` | inline | `N * size_of::<T>()` | none; the length is in the type |
| `&[T]` / `&str` | borrowed | 16 bytes | ptr, len |
| `Vec<T>` / `String` | heap, owned | 24 bytes | ptr, cap, len |

- A slice has no capacity field, so it physically can't grow.
- **Bounds checks** are often free: LLVM removes them when it can prove the index is in range, iterators never need them, and an `assert!` before a loop can eliminate them. `get_unchecked` is the last resort.
- `slice::from_raw_parts` turns C memory into a Rust slice. It's `unsafe` because you vouch for the pointer, length, alignment and lifetime.
- **Strings:**
  - A C string is NUL-terminated, `strlen` is O(n) and the encoding is unknown.
  - A Rust `str` is a pointer plus a length, always valid UTF-8, and can contain zeros.
  - `CStr` and `CString` (and `c"..."` literals) bridge between the two.
  - `OsStr` and `Path` hold OS strings that aren't guaranteed to be UTF-8.

**Exercises:**
- Predict the `size_of` of various handles.
- Compare bounds-check assembly with and without an `assert!` before the loop.
- Write `strlen` by hand.
- Overread a slice and watch Miri catch it.

> **Rule:** A pointer without a length is half a fact. Rust's slices carry the other half everywhere.

## Module 5 — Layout: size, alignment, padding and `repr`

A type's layout is three things: its size, its alignment and each field's offset. Two rules generate all of it:
- A field starts at a multiple of its own alignment.
- A type's size rounds up to a multiple of its alignment.

| `repr` | Meaning | Use for |
|--------|---------|---------|
| (default) `Rust` | no layout promise; fields may reorder | everything else |
| `C` | declaration order, C padding rules | FFI, and on-disk or on-chain structs |
| `packed` | alignment 1, no padding | wire formats; you can't take `&field` (error E0793) |
| `align(N)` | raise the alignment | separating data onto different cache lines |
| `transparent` | same layout as the inner field | newtypes that cross FFI |
| `u8` etc. on an enum | fixed tag type | C-style enums |

- **Enums are tagged unions** where `match` forces you to check the tag. A raw `union` read is `unsafe`.
- **Niches** let the compiler store an enum's tag in bit patterns the payload can never use:
  - `Option<&T>` is 8 bytes, the same as `&T`, because `None` uses the null pattern.
  - `Option<NonZeroU32>` is 4 bytes, the same as `NonZeroU32`, because zero is free.
  - `Option<u32>` is 8 bytes, because every `u32` pattern is valid and a separate tag is needed.

  ↔ SPL's `COption<Pubkey>` is 36 bytes on-chain but `Option<Pubkey>` is 33 bytes in memory. Memory layout is the compiler's choice; wire layout is yours.
- **Zero-sized types** (`()` and `PhantomData`) cost nothing at run time.
- **Padding bytes are uninitialised.** Viewing a padded struct as `&[u8]` is UB.

**Exercises:**
- Compute struct offsets and reorder fields to shrink a struct.
- Predict niche sizes for several `Option` and `Result` types.
- Dump an enum's bytes to find its tag.
- Fix a `packed` field reference.

> **Rule:** Memory layout is the compiler's choice unless you say `repr(C)`. Anything that crosses a boundary (FFI, disk, network, chain) needs a layout you chose.

## Module 6 — Dynamic memory: build `malloc`'s client, then `malloc`

| C bug | What stops it in Rust |
|-------|-----------------------|
| Leak | `Drop` (leaks are still *safe*: `mem::forget`, `Rc` cycles) |
| Double free | a single owner means exactly one drop |
| Use-after-free | the borrow checker |
| Uninitialised read | `MaybeUninit<T>` |
| Buffer overflow | bounds-checked slices |

- **Rust's allocation API is sized.** `dealloc(ptr, layout)` needs the exact `Layout` you allocated with, so the allocator needs no hidden header.
- **Build `MyVec<T>`** with a pointer, a capacity and a length. Use `ptr::write` into raw memory, not `=`, because assignment would drop the uninitialised garbage that was there.
- **`#[global_allocator]`** lets you swap in your own allocator. Start with one that counts allocations.
- **The bump allocator:** allocating rounds the offset up with `(offset + align - 1) & !(align - 1)`, and freeing does nothing. ↔ A Solana program's default heap is a 32 KiB bump allocator that never frees.

**Exercises:**
- Finish `MyVec` with `pop`, `Drop` and `Deref`.
- Break `Drop` on purpose: forget to drop the elements, then drop them twice.
- Measure `String` growth with the counting allocator.
- Make `MyVec` handle zero-sized types.
- Install a bump allocator as the global allocator.

> **Rule:** Allocation is a contract: you get memory, you owe back its exact layout, exactly once. Ownership is that contract enforced.

## Module 7 — Functions, stack frames, the ABI and dispatch

- **System V calling convention:** arguments in `rdi…r9`, the return value in `rax`, and a hidden pointer for large return values. `extern "Rust"` is deliberately unspecified so the compiler can optimise it; `extern "C"` pins the C convention.
- **Stack frames:** at `opt-level=0` you can see the prologue, body and epilogue. At `opt-level=3`, leaf functions often have no frame at all. Inlining matters more for performance than anything else in this module.
- **Four ways to pass behaviour:**

| Mechanism | In memory | Dispatch |
|-----------|-----------|----------|
| `fn` pointer | one code address | indirect call |
| Closure (`impl Fn`) | a struct of its captures | direct call, usually inlined |
| Generic / `impl Trait` | one compiled copy per type | direct call |
| `dyn Trait` | fat pointer: data pointer plus vtable pointer | indirect call through the vtable |

- **A vtable** holds the drop glue, size, alignment and one pointer per method. This layout is common in practice but not a stable guarantee.

**Exercises:**
- Find each argument's register in the assembly.
- Find the hidden return pointer.
- Compare the assembly for a generic loop and a `dyn` loop.
- Hand-build a trait object, C style.
- Measure closure sizes.

> **Rule:** Every call answers two questions: which code, and whose data. Generics answer at compile time; `dyn` answers at run time through a table.

## Module 8 — FFI: talking to C in both directions

- **Calling C:** declare the functions in an `unsafe extern` block (required in edition 2024). Use the `cc` crate in `build.rs` to compile your own C files, and `bindgen` for large headers.
- **Being called from C:** use `#[unsafe(no_mangle)] extern "C" fn`, build a `cdylib` or `staticlib`, and generate a header with `cbindgen`.
- **Ownership across the line:** `Box::into_raw`/`from_raw` and `CString::into_raw`/`from_raw` hand objects over and take them back; `CStr::from_ptr` borrows without taking ownership. **Memory is freed by the allocator that allocated it.**
- **Panics:** a panic that unwinds out of `extern "C"` aborts the process. Use `catch_unwind` and return an error code instead. Callbacks follow C's pattern: an `extern "C" fn` plus a `*mut c_void` user-data pointer, with a small "trampoline" that casts it back to your closure.
- **Opaque handles:** hide the layout and expose only functions.

**Exercises:**
- Wrap a C ring buffer in a safe Rust type.
- Call `qsort` with a Rust comparator, including one that captures a key.
- Export a `Counter` to C and check it with AddressSanitizer.
- Catch a panic at the boundary.
- Write the `# Safety` section for your wrapper.

> **Rule:** The boundary is `unsafe` once, in one place. Wrap it, document what you assume, and let everything above be safe.

## Module 9 — Bytes on the wire: parsing binary formats

Casting a `char*` buffer to a `struct*` is fast, and it has been the source of a generation of exploits. Rust has two disciplined versions of the same idea:

- **Strategy A, copy out:** slice the buffer and convert each field with `from_le_bytes`. It's always sound and is usually just as fast. **Start here.**
- **Strategy B, zero-copy:** reinterpret `&[u8]` as `&T` without copying. This is sound only if all four hold:
  - `T` is `repr(C)`,
  - `T` has no padding,
  - every bit pattern is a valid `T`,
  - the buffer is aligned for `T`.

  `bytemuck` and `zerocopy` check these conditions. ↔ Anchor's `zero_copy` is `bytemuck` underneath.
- **A safe TLV parser** returns `Option` at every step, and its entries borrow from the buffer, so it's zero-copy with no `unsafe`. ↔ Token-2022's `StateWithExtensions::unpack` works the same way.
- **Length fields are attacker input.** Check them before you slice or allocate.
- **Fuzz** with `cargo fuzz`.

**Exercises:**
- Turn the TLV parser into an iterator.
- Hand-parse a real SPL mint.
- Read a struct zero-copy with `bytemuck`.
- Do a misaligned read under Miri.
- Fuzz the parser, then plant an off-by-one bug and time how long the fuzzer takes to find it.

> **Rule:** Bytes from outside are hostile until proven otherwise. Check lengths before slicing, copy out by default, and go zero-copy only when a type-level proof exists.

## Module 10 — Caches and performance

| Level | Size | Latency |
|-------|------|---------|
| L1 | 32–48 KiB per core | ~1 ns |
| L2 | 1–2 MiB per core | ~3–5 ns |
| L3 | tens of MiB, shared | ~10–40 ns |
| DRAM | GiBs | ~80–100 ns |

- Memory moves in **64-byte cache lines**. Sequential access is cheap; pointer chasing pays full latency at every hop.
- **Layout is performance:**
  - Walking a grid in row order is much faster than in column order.
  - A struct of arrays can beat an array of structs when a loop reads only some fields.
  - A `Vec` beats a linked list for almost everything.
  - Smaller types fit more values per cache line.
- **Branch prediction:** a mispredicted branch costs about 15 cycles, so sorted input can make the same code run several times faster.
- **Where Rust has an edge over C:** `&mut` guarantees no aliasing, so LLVM can vectorise; iterators drop bounds checks.
- **Measure properly:** use release builds, `criterion`, `black_box`, `perf stat -e cache-misses,branch-misses` and `-C target-cpu=native`.

**Exercises:**
- Compare row and column traversal.
- Compare a linked list with a `Vec`.
- Convert an array of structs to a struct of arrays and look for `ymm` registers in the assembly.
- Run the sorted vs unsorted branch experiment.
- Profile a loop from your own code.

> **Rule:** The fastest memory access is the one already in cache. Design data for the loop that reads it, and measure before believing.

## Module 11 — Concurrency, atomics and memory ordering

- **`Send`** means a value can move to another thread. **`Sync`** means `&T` can be shared between threads. The compiler derives both, and `thread::spawn` requires `Send`, which turns data races into type errors.

| Type | Send | Sync |
|------|------|------|
| `Rc<T>` | no | no |
| `Arc<T>` | if `T: Send + Sync` | if `T: Send + Sync` |
| `RefCell<T>` | if `T: Send` | no |
| `Mutex<T>` | if `T: Send` | if `T: Send` |

- **Scoped threads** (`thread::scope`) can borrow from the parent's stack.
- **Atomics:** `fetch_add` on x86 compiles to a single `lock xadd`.
- **Memory ordering:**
  - `Relaxed` makes the operation atomic and nothing more.
  - A `Release` store paired with an `Acquire` load publishes data from one thread to another.
  - `SeqCst` adds a single global order.

  x86 hides ordering bugs that show up on ARM, so reason about orderings and check them with **`loom`**.
- **Build a spinlock** on `UnsafeCell` with `unsafe impl Sync`. `UnsafeCell` is the only legal way to mutate through a `&`.

**Exercises:**
- Read the error you get from sending an `Rc` to another thread.
- Write a `Relaxed` counter.
- Use `loom` to find and fix a publish bug with Release/Acquire.
- Make the spinlock panic-safe with an RAII guard.
- Measure false sharing, then fix it with `repr(align(128))`.

**Read alongside:** Mara Bos, *Rust Atomics and Locks*, chapters 1–4.

> **Rule:** `Send` and `Sync` turn data races into type errors. Below them, an ordering is a promise about visibility; choose the weakest one you can prove correct.

## Module 12 — The OS boundary: syscalls, file descriptors, virtual memory

- **Syscalls:** the `syscall` instruction takes its number in `rax` and arguments in `rdi, rsi, rdx, r10, r8, r9`, and returns a negative value on error. Use `strace` to see them.
- **File descriptors get owners:** `OwnedFd` closes on drop, `BorrowedFd<'a>` is a checked borrow, and `RawFd` is a bare integer for FFI only. This is the same three-way split as `Box`, `&` and `*mut`.
- **Virtual memory:** 4 KiB pages and per-process page tables explain ASLR, guard pages and segfaults.
- **`mmap`** (via the `memmap2` crate) gives you a file as a `&[u8]`. It's `unsafe` because another process can change the file underneath you.
- **Processes:** `std::process::Command` covers fork, exec and wait, with pipes via `Stdio::piped()`. `signal-hook` handles signals safely.
- ↔ Solana programs also reach the runtime only through syscalls (`sol_log_`, `sol_invoke_signed_c`).

**Exercises:**
- Run `strace` on a Rust and a C hello-world and compare.
- Write `cat` three ways.
- Close the same fd twice and see what happens.
- Count lines with `mmap` vs `BufReader`.
- Build a pipeline without a shell.

> **Rule:** The kernel owns the real resources; a descriptor is a claim ticket. Give every ticket exactly one owner, and it'll be returned exactly once.

## Module 13 — `no_std` and bare metal

- **`std` is three layers:**
  - `core` needs nothing.
  - `alloc` needs a global allocator.
  - `std` adds the OS.

  `#![no_std]` leaves you with `core`. Add `extern crate alloc` and a global allocator to get `alloc` back.
- **Without the OS, three jobs become yours:**
  - the entry point (`#![no_main]` and `_start`),
  - a `#[panic_handler]`, with `panic = "abort"`,
  - I/O, through raw syscalls or hardware registers.
- **Memory-mapped I/O** uses `ptr::write_volatile`. You can try it on a Raspberry Pi Pico, a micro:bit or QEMU.
- ↔ **Solana SBF is a constrained bare-metal target**, and each of its limits maps to a module:

| SBF constraint | Module |
|----------------|--------|
| Entrypoint gets one `*mut u8` input buffer | 3 |
| 32 KiB bump heap that never frees | 6.5 |
| 4 KiB stack frame limit | 2 |
| Zero-copy account data | 5, 9 |
| Runtime syscalls | 12 |
| Compute-unit cost | 10 |

**Exercises:**
- Build a freestanding binary that prints `hi` and exits with status 7.
- Port the TLV parser to `no_std`.
- Add a bump allocator and a `Vec`.
- Overflow an SBF stack frame.
- Write `// SAFETY:` comments for the `solana-program` entrypoint.

> **Rule:** `std` is a convenience layer over `core`. Remove it and nothing about the machine changes; you just see it.

---

## Capstone projects

Pick **at least three**. Before writing code for any of them, write one page covering the data layout, the invariants, and where each `unsafe` block will go and what it assumes.

| Project | Modules | Done when |
|---------|---------|-----------|
| **1. A real malloc.** A free-list allocator over `mmap`ed pages, with size classes, first-fit, splitting and coalescing | 5, 6, 11, 12 | A real crate's `cargo test` passes with it installed, under ASan, and a counter shows memory being reused |
| **2. An open-addressing hash map** with linear or Robin Hood probing, `MaybeUninit` slots, tombstones and resizing | 4, 5, 6, 10 | Property tests against `std::HashMap` pass under Miri, and a benchmark comes with an explanation of the gap |
| **3. A mini shell** with pipelines, `<`/`>` redirects, `cd`/`exit` and Ctrl-C handling | 8, 12 | `cat file \| grep x \| wc -l` works with no leaked file descriptors |
| **4. A lock-free SPSC ring buffer** with power-of-two capacity and head and tail on separate cache lines | 5, 10, 11 | The `loom` model passes, it beats `std::sync::mpsc`, and you can explain why |
| **5. A zero-copy Solana program** with no Anchor: raw entrypoint, manual account parsing and zero-copy state | 3, 5, 9, 13 | The same tests pass, compute units are measured before and after, and every `unsafe` block has a defensible `// SAFETY:` comment |

Keep a short log for each capstone: what you expected, what the machine did, and the rule you took away.

## Suggested pacing

About 12 weeks at 6–8 hours a week:

| Weeks | Work |
|-------|------|
| 1–2 | Modules 0–2 |
| 3–4 | Modules 3–5 |
| 5 | Module 6 and capstone 2 |
| 6–7 | Modules 7–9 |
| 8 | Module 10 |
| 9–10 | Module 11 and capstone 4 |
| 11 | Modules 12–13 |
| 12 | One of capstones 1, 3 or 5 |

## Reading list

| Resource | Pair with |
|----------|-----------|
| Bryant & O'Hallaron, *Computer Systems: A Programmer's Perspective* (CS:APP) | the whole course |
| *The Rustonomicon* | Modules 3, 5, 6 |
| *The Rust Reference*: Type layout | Module 5 |
| Mara Bos, *Rust Atomics and Locks* | Module 11 |
| Jon Gjengset, *Rust for Rustaceans* and the Crust of Rust streams | Modules 3, 6, 7, 11 |
| Philipp Oppermann, *Writing an OS in Rust* | Modules 12, 13 |
| *The Embedded Rust Book* | Module 13 |
| Luca Palmieri, *Zero To Production in Rust* | writing tests you trust |

## The rules, in one place

| Module | Rule |
|--------|------|
| 0 Toolkit | Source is a request; assembly is what happened. |
| 0.5 Assembly | Result on the left, arguments in `rdi, rsi`, answer in `rax`. |
| 1 Integers | A type is an interpretation of bits. Decide width, conversion and overflow on purpose. |
| 2 Memory map | A value's region decides its lifetime; Rust lifetimes write that down. |
| 3 Pointers | A reference is a pointer plus a proof. `unsafe` means you write the proof. |
| 4 Slices | A pointer without a length is half a fact. |
| 5 Layout | Anything crossing a boundary needs a layout you chose (`repr(C)`). |
| 6 Allocation | You owe back the exact layout, exactly once. Ownership enforces it. |
| 7 Calls | Every call answers "which code, whose data"; generics at compile time, `dyn` at run time. |
| 8 FFI | The boundary is `unsafe` once, in one place. Wrap it and document assumptions. |
| 9 Parsing | Outside bytes are hostile. Check lengths first; copy out by default. |
| 10 Caches | Design data for the loop that reads it; measure before believing. |
| 11 Concurrency | `Send`/`Sync` make races type errors; pick the weakest ordering you can prove. |
| 12 OS | A descriptor is a claim ticket; give it exactly one owner. |
| 13 `no_std` | Remove `std` and nothing about the machine changes; you just see it. |
