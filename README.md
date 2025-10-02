# sub0bfuscate

## Turn *all* arithmetic operations into subtractions of 0.0 and -0.0 and an additional constant 0

## Inspiration
[Tom Murphy VII](http://tom7.org/nand/nand.pdf) and [Orson Peters](https://orlp.net/blog/subtraction-is-functionally-complete/) (the basic implementation is continued from his code) for the original research (although it may have been known before, it was new to me) and [u/MyOthrUsrnmIsABook](https://www.reddit.com/user/MyOthrUsrnmIsABook/) for the not-quite-serious idea of actually using it. I'm just trying to be the dummy who actually follows through with it.

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
It is well established in computer science that the only boolean operators that are functionally complete by themselves are NAND and NOR. This means that any boolean function can be expressed using _only_ a combination of NAND or NOR gates on their own. However, sets of other logical operators can also be functionally complete. For example, the set {AND, NOT} is functionally complete. This means that any boolean function can be expressed using only AND ($\land$) and NOT ($\neg$) gates. 

Now, since we found out that subtraction of the two distinct IEEE754 floating point zeros {0.0, -0.0} produces an IMPLY gate, we can check for a minimal set involving IMPLY that is functionally complete. 

### Functional Completeness of {IMPLY, 0}
The set 

$$ 
\{\rightarrow, 0\} 
$$

 (read: IMPLICATION and a constant FALSE) is functionally complete. Interestingly, that is not the case for $\{\rightarrow, 1\}$. When you have the IMPLY gate and a constant, it is possible to build other gates from which you can derive any Boolean function. We can prove this by building the set $\{\land, \neg\}$ from $\{\rightarrow, 0\}$ since we already know the former to be complete.    


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

We have now expressed $\{\land, \neg\}$ ($AND$ and $NOT$) using only $\{\rightarrow, 0\}$.

This proves that $ \{ \rightarrow, 0 \} $ is indeed functionally complete because we can derive the necessary logical operations $ \{ \land, \neg \} $ from it, which can directly be used to construct any Boolean function.  
Since computers are built using boolean logic, this in turn means that any arithmetic operation can be expressed using only subtraction of the two distinct IEEE754 floating point zeros (with the technical nitpick that you need an additional constant 0 as input, which is still kind of beautiful because the only constant you need is still a zero).


## Computing
We've established that we _can_ build _EVERYTHING_ from subtraction of the two IEEE754 floating point zeros and an additional constant 0. But how do we actually do it?

Let's start by implementing our logic gates.

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

Then we proceed to build `soft integers` (like soft floats, but turned on its head) on top of our logic gates. That gives us basic arithmetic operations on integers.

**STOP!** I hear you say at this point. But oh no, friend, we're not done yet. We still need to implement floating point numbers. And since we are building soft floats on top of already soft integers, they must surely be called `softer floats`!


After doing all this, we can now potentially replace _EVERY_ arithmetic operation in _any_ code with a combination of subtractions of the two IEEE754 floating point zeros and an additional constant 0.

## Status
The obfuscator part is not fleshed out yet, I'm still working on the basic building blocks of arithmetic.
It will probably involve rummaging around in LLVM IR, replacing arithmetic ops with function calls.


