# feruca – Unicode collation in Rust

feruca is a basic implementation of the
[Unicode Collation Algorithm](https://unicode.org/reports/tr10/) in 100% safe
Rust (outside of the tests module). It is current with Unicode **version 14.0**.
The name of the library is a portmanteau of Ferris 🦀 and UCA.

I mean a few things by "basic implementation." First, I don't expect that this
is highly performant. My rough attempts at benchmarking suggest that feruca is
on the order of 25–50x slower than `ucol` from
[icu4c](https://github.com/unicode-org/icu). But my initial priority was to pass
the official
[conformance tests](https://www.unicode.org/Public/UCA/latest/CollationTest.html).
feruca also passes the conformance tests for the
[CLDR](https://github.com/unicode-org/cldr) root collation order (more on this
below).

Second, there is not yet support for tailoring, beyond being able to choose
between the Default Unicode Collation Element Table (DUCET) and the default
variation from CLDR. (You can additionally choose between the "non-ignorable"
and "shifted" strategies for handling variable-weight characters.) Adding
further support for tailoring is a near-term priority.

Third, the library has at present only one public function: `collate`, which
accepts two string references (plus a `CollationOptions` struct), and returns an
`Ordering`. That is, you can pass `collate` to the standard library function
`sort_by` (see "Example usage").

For many people and use cases, UCA sorting will not work properly without being
able to specify a certain locale. That being said, the CLDR root collation order
is already quite useful. When calling the `collate` function, you can pass
default options (see below), which specify the use of the CLDR table with the
"shifted" strategy. I think this is a good starting point.

## Example usage

```rust
use feruca::{collate, CollationOptions};

fn main() {
    let mut uca = [
        "چنگیز",
        "Éloi",
        "Ötzi",
        "Melissa",
        "صدام",
        "Mélissa",
        "Overton",
        "Elrond",
    ];

    let mut naive = uca;

    uca.sort_by(|a, b| collate(a, b, CollationOptions::default()));
    naive.sort();

    for item in uca {
        println!("{}", item);
    }
    // Éloi
    // Elrond
    // Melissa
    // Mélissa
    // Ötzi
    // Overton
    // چنگیز
    // صدام

    // Add an empty line between the lists (in case you actually run this)
    println!();

    for item in naive {
        println!("{}", item);
    }
    // Elrond
    // Melissa
    // Mélissa
    // Overton
    // Éloi
    // Ötzi
    // صدام
    // چنگیز
}
```
