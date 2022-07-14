# feruca – Unicode collation in Rust

feruca is a basic implementation of the
[Unicode Collation Algorithm](https://unicode.org/reports/tr10/) in Rust. It's
current with Unicode **version 14.0**. The name of the library is a portmanteau
of Ferris 🦀 and UCA.

Outside of the tests module, no `unsafe` is used directly in this library. It
relies on the well-vetted [bstr](https://github.com/BurntSushi/bstr) to accept
input (in the form of either `&str` or `&[u8]`), to perform UTF-8 validation,
and to generate a list of Unicode scalar values, which can then be processed for
collation. The idea is to be tolerant of input that may not be entirely kosher
UTF-8.

In describing feruca as a "basic implementation," I have a few things in mind.
First, I don't expect that it will win any awards for performance. My rough
attempts at benchmarking suggest that this is on the order of 10–20x slower than
`ucol` from [icu4c](https://github.com/unicode-org/icu). But my initial priority
was to pass the official
[conformance tests](https://www.unicode.org/Public/UCA/latest/CollationTest.html).
feruca also passes the conformance tests for the
[CLDR](https://github.com/unicode-org/cldr) root collation order (more on this
below).

Second, there is not yet support for tailoring, beyond being able to choose
between the Default Unicode Collation Element Table (DUCET) and the default
variation from CLDR. (You can additionally choose between the "non-ignorable"
and "shifted" strategies for handling variable-weight characters.) Adding
further support for tailoring is a near-term priority.

Third, the library has effectively\[0\] just one public function: `collate`,
which accepts two string references or byte slices (plus a `CollationOptions`
struct), and returns an `Ordering`. That is, you can pass `collate` to the
standard library function `sort_by` (see "Example usage").

For many people and use cases, UCA sorting will not work properly without being
able to specify a certain locale. That being said, the CLDR root collation order
is already quite useful. When calling the `collate` function, you can pass
default options (see below), which specify the use of the CLDR table with the
"shifted" strategy. I think this is a good starting point.

\[0\]: There is also a variant form, `collate_no_tiebreak`, which will return
`Ordering::Equal` for any two strings that produce the same UCA sort key. (The
normal version will fall back on byte-value comparison in such cases.)

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

    // Add a line of space (in case you run this verbatim)
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
