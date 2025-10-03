# sub0bfuscate

Turn *all* arithmetic operations into subtractions of 0.0 and -0.0 (and an additional constant 0) for code obfuscation, I guess.

## Inspiration

[Tom Murphy VII](http://tom7.org/nand/nand.pdf) and [Orson Peters](https://orlp.net/blog/subtraction-is-functionally-complete/) (the basic implementation is continued from his code) for the original research (although it may have been known before, it was new to me). I just thought he didn't take it far enough for the full comedic effect.

## Principle

### Floating Point Zeros

IEEE754 floating point numbers have two zeros: 0.0 and -0.0 They are distinct values, but they compare equal (read more at [Orson Peters' Blog](https://orlp.net/blog/subtraction-is-functionally-complete/)). This means that the following is true:

$$
\begin{align*}
+0.0 - +0.0 &= +0.0 \\
0.0 - (-0.0) &= +0.0 \\
-0.0 - +0.0 &= -0.0 \\
-0.0 - (-0.0) &= +0.0
\end{align*}
$$

Now, let's say $-0.0$ is $FALSE$ and $0.0$ is $TRUE$, we can make a truth table:

| P | Q | P - Q |
|:-:|:-:|:-----:|
| T | T |   T   |
| T | F |   T   |
| F | T |   F   |
| F | F |   T   |

This is the truth table for the $IMPLICATION$ operation (aka. $IMPLY$ or $\rightarrow$).
Interesting, but not as useful as it could be if it was an outright NAND or NOR gate.
Let's explore what can be built out of $IMPLY$s

### Functional Completeness

It is well established in computer science that the only boolean operators that are functionally complete by themselves are NAND and NOR. This means that any boolean function can be expressed using *only* a combination of NAND or NOR gates on their own. However, sets of other logical operators can also be functionally complete. For example, the set {AND, NOT} is functionally complete. This means that any boolean function can be expressed using only AND ($\land$) and NOT ($\neg$) gates.

Now, since we found out that subtraction of the two distinct IEEE754 floating point zeros {0.0, -0.0} produces an IMPLY gate, we can check for a minimal set involving IMPLY that is functionally complete.

### Functional Completeness of {IMPLY, 0}

The set

$$
\lbrace\rightarrow,0\rbrace
$$

 (read: IMPLICATION and a constant FALSE) is functionally complete. Interestingly, that is not the case for $\lbrace\rightarrow,1\rbrace$. When you have the IMPLY gate and a constant, it is possible to build other gates from which you can derive any Boolean function. We can prove this by building the set $\lbrace\land,\neg\rbrace$ from $\lbrace\rightarrow,0\rbrace$ since we already know the former to be complete.

Given that the IMPLY operation can be defined as:

$$
P \rightarrow Q = \neg P ~ \lor ~ Q
$$

We have a particular scenario when we replace Q with a constant FALSE (0):

$$
P \rightarrow 0 = \neg P ~ \lor ~ 0
$$

Since OR with FALSE does not change the value of the other operand:

$$
P \rightarrow 0 = \neg P
$$

Therefore, we can use an IMPLY gate with the second input fixed to 0 (FALSE) to implement the NOT gate.

Now, let's find a way to express AND using NOT and IMPLY. Using De Morgan’s laws, AND can be written in terms of NOT and OR:

$$
P \land Q = \neg (\neg P \lor \neg Q)
$$

Since we have already found a way to express $NOT$ using $\{\rightarrow, 0\}$, we just need to find a way to express $OR$ in terms of at most $\{\rightarrow, 0\}$. Luckily $OR$ can be implemented with $\{\rightarrow, \neg\}$ in the following way:

$$
P \lor Q = \neg P \rightarrow Q
$$

We can easily verify this in the truth table

| P | Q | $\neg P$ | $P \lor Q = \neg P \rightarrow Q$ |
|:-:|:-:|:-----:|:----------:|
| T | T |   F   |     T      |
| T | F |   F   |     T      |
| F | T |   T   |     T      |
| F | F |   T   |     F      |

So, to recap:

$$
\begin{align}
\neg P &= P \rightarrow 0 \\
P \lor Q &= \neg P \rightarrow Q
\end{align}
$$

Substituting 1 and 2 into the AND expression we got from De Morgan's laws in terms of NOT and OR:

$$
\begin{align*}
P \land Q &= \neg (\neg P \lor \neg Q) \\
P \land Q &= \neg ((P \rightarrow 0) \rightarrow Q) \\
P \land Q &= ((P \rightarrow 0) \rightarrow Q) \rightarrow 0) \\
\end{align*}
$$

We have now expressed $\lbrace \land, \neg\rbrace$ ($AND$ and $NOT$) using only $\{\rightarrow, 0\}$.

This proves that $\lbrace\rightarrow,0\rbrace$ is indeed functionally complete because we can derive the necessary logical operations $\lbrace\land,\neg\rbrace$ from it, which can directly be used to construct any Boolean function.  
Since computers are built using boolean logic, this in turn means that any arithmetic operation can be expressed using only subtraction of the two distinct IEEE754 floating point zeros (with the technical nitpick that you need an additional constant 0 as input, which is still kind of beautiful because the only constant you need is still a zero).

## Computing

We've established that we *can* build *EVERYTHING* from subtraction of the two IEEE754 floating point zeros and an additional constant 0. But how do we actually do it?

Let's start by implementing our logic gates.  

(Thanks to [Orson Peters](https://orlp.net/blog/subtraction-is-functionally-complete/) for the initial code)

```Rust
type Bit = f32;
const ZERO: Bit = -0.0;
const ONE: Bit = 0.0;

fn not(x: Bit) -> Bit { ZERO - x }
fn or(a: Bit, b: Bit) -> Bit { a - not(b) }
fn and(a: Bit, b: Bit) -> Bit { not(or(not(a), not(b))) }
fn xor(a: Bit, b: Bit) -> Bit { or(and(not(a), b), and(a, not(b))) }
```

Then we can implement a standard full-adder (and later a multiplier) on top of that.

```Rust
fn adder(a: Bit, b: Bit, c: Bit) -> (Bit, Bit) {
    let s = xor(xor(a, b), c);
    let c = or(and(xor(a, b), c), and(a, b));
    (s, c)
}
```

Then, using these primitives, we proceed to build `soft integers` (like soft floats, but turned on its head) on top of our logic gates. That gives us basic arithmetic operations on integers.

```Rust
struct SoftU8([Bit; 8]); // 8-bit unsigned integer
// similarly SoftU23/24, SoftU32
fn softu8_add(a: SoftU8, b: SoftU8) -> SoftU8 { ... }
```

**STOP!** I hear you say at this point. But no, friend, we're not done yet. You see, Orson seems to be a reasonable Person satisfied with his result.  
But we cannot rest until we have re-implemented soft floating point numbers on top of soft integers on top of hard floats to complete the cycle and put our minds at ease.
And since we are building soft floats on top of already soft integers, they must surely be called `softer floats`!

```Rust
pub struct SofterF32 {
    pub sign: Bit,
    pub exponent: SoftU8,  // stored exponent (no bias removed)
    pub fraction: SoftU23, // stored 23 bits (no implicit 1 here)
}
pub fn softerf32_add(a: SofterF32, b: SofterF32) -> SofterF32 { ... }
```

After doing all this, we can now potentially replace *EVERY* addition in *any* code with their evil twin - a combination of subtractions of the two IEEE754 floating point zeros (and an additional constant 0).

Now, if our `SofterF32` implementation were to adhere to the IEEE754 standard, we could go further and implement another layer of *even softer* integers on top of the soft floats, but now even I begin to think we're pushing it.


## Looking at the Generated Code
Now, let's see what the generated LLVM IR looks like for the `softerf32_add` function (never inlined, and with the `softerf32_add` symbol preserved for easy lookup).

First, we build a more portable release build for better comparison:

```bash
RUSTFLAGS="-C target-cpu=x86-64 -C opt-level=3" \
cargo build --release --target x86_64-unknown-linux-gnu
```

Then, we can use `llvm-objdump` to generate the disassembly for the `softerf32_add` function and filter for the number of lines in the disassembly pertaining to the `softerf32_add` function, which gives us an idea of its complexity.  

```bash
llvm-objdump -d -C --no-show-raw-insn target/release/sub0bfuscate | awk '
/^[0-9a-f]+ <[^>]+>:$/ {
  name=$0
  sub(/^[0-9a-f]+ +</,"",name); sub(/>:.*/,"",name)
  keep = (name ~ /^sub0bfuscate::softcore::/) || (name ~ /^softerf32_/)
  next
}
keep && /^[[:space:]]*[0-9a-f]+:\s/ { print }
' | wc -l

```

On my machine, this outputs `1719` lines.  
So, approximating one instruction per line, we can say that the `softerf32_add` function requires about 1719 instructions. This is quite a lot compared to a normal floating point addition, which typically uses just a handful of instructions for loading params, adding, and storing the result.


Sanity check which functions were filtered: 

```bash
llvm-objdump -d -C --no-show-raw-insn target/release/sub0bfuscate | awk '
/^[0-9a-f]+ <[^>]+>:$/ {
  name=$0
  sub(/^[0-9a-f]+ +</,"",name); sub(/>:.*/,"",name)
  if (name ~ /^sub0bfuscate::softcore::/ || name ~ /^softerf32_/) print name
}'
```

Which should show (although I am not sure why `softu24_add` is not included, maybe it got inlined anyway due to visibility or monomorphization):

```
sub0bfuscate::softcore::bitops::not::hcd535935072e2f86
sub0bfuscate::softcore::bitops::or::h74512b828b43446f
sub0bfuscate::softcore::bitops::and::h28001bf75618b28d
sub0bfuscate::softcore::bitops::xor::h5627245ac43baf3c
sub0bfuscate::softcore::bitops::adder::hcba00d0bffbcf38f
sub0bfuscate::softcore::softerfloat::to_softerf32::hff2d59c252edbd3c
sub0bfuscate::softcore::softerfloat::from_softerf32::hd8379f866c8cfe79
softerf32_add
```

If you want to see the full disassembly of the `softerf32_add` function, run:

```bash
llvm-objdump -d -C --no-show-raw-insn target/release/sub0bfuscate \
  | sed -n '/<softerf32_add>/,/<.*>:/p'
```

This will output the disassembly (AT&T/GNU syntax) for the `softerf32_add` function, which can be analyzed to see how the arithmetic operations have been transformed into subtractions of 0.0 and -0.0.
You will see a lot of low level instructions like `xorps` (zero register), `movss` (load 0.0 from .rodata, this is heavily used), `pxor` (zeroing integer XOR), `subss` (the actual subtraction instruction for our NOT gate `not(x) = (-0.0) -x`), etc., which are the most relevant floating point operations used by our assembly.

You may now ponder the comic absurdity of scrolling past all those `movss` instructions in the assembly.

## Status

This is not an actually usable obfuscator yet, maybe it never will - you know how it is with these tangential projects. 
It will probably involve rummaging around in LLVM IR, replacing arithmetic ops with calls to the appropriate functions in this crate, and then letting LLVM optimize the result.  
I am not sure I am qualified to do that.
