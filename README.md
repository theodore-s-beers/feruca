# feruca – Unicode collation in Rust

feruca is a basic implementation of the
[Unicode Collation Algorithm](https://unicode.org/reports/tr10/) in Rust. It's
current with Unicode **version 15.0**. The name of the library is a portmanteau
of Ferris 🦀 and UCA.

No `unsafe` is used directly in this library: `#![forbid(unsafe_code)]`. It
relies on the well-vetted [bstr](https://github.com/BurntSushi/bstr) to accept
input (in the form of either `&str` or `&[u8]`), to perform UTF-8 validation,
and to generate a list of Unicode scalar values, which can then be processed for
collation. The idea is to be tolerant of input that may not be entirely kosher
UTF-8.

In describing feruca as a "basic implementation," I have a few things in mind.
**First**, I don't expect that it will win any awards for performance. My
[rough attempts](https://github.com/theodore-s-beers/feruca-benchmarks) at
benchmarking suggest that this is on the order of 3x slower than `ucol` from
[icu4c](https://github.com/unicode-org/icu). (On the other hand, that isn't as
bad as one might imagine, considering the incredible degree of optimization
achieved in the ICU C libraries. And I have found that the performance of the
new [icu4x](https://github.com/unicode-org/icu4x) collator, also implemented in
Rust, does not yet match that of feruca.) My initial priority was to pass the
official
[conformance tests](https://www.unicode.org/Public/UCA/latest/CollationTest.html).
feruca also passes the conformance tests for the
[CLDR](https://github.com/unicode-org/cldr) root collation order.

**Second**, support for tailoring is minimal (so far). You can choose between
two tables of character weights: the Default Unicode Collation Element Table
(DUCET), or the CLDR variation thereof. The CLDR table then becomes the starting
point for actual collation tailoring based on language/locale. I have added only
one tailoring, intended for use with Arabic-script languages. It shifts letters
in the Arabic script so that they sort before the Latin script. This is enough
for my own work with Persian and Arabic texts. The CLDR table in its unmodified
form—i.e., the root collation order—works out-of-the-box for several other
languages. I do plan to add more tailorings, but it will be a gradual process,
and driven by demand. Realistically, feruca will never have the kind of
all-encompassing, flexible support for tailoring that is provided by ICU. My
feeling is that there's a place for less sophisticated solutions, with simpler
APIs, smaller dependency trees, etc. (If you have thoughts on this, I would be
interested in hearing them.)

Apart from locale tailoring, you can choose between the "non-ignorable" and
"shifted" strategies for handling variable-weight characters—with the latter
being the default.

**Third**, this library has effectively\[0\] just one public method, `collate`,
belonging to a struct, `Collator`, which sets a few options. `collate` accepts
two string references or byte slices, and returns an `Ordering` value. It is
designed to be passed as a comparator to the standard library method `sort_by`
(or `sort_unstable_by`). See "Example usage" below.

For many people and use cases, UCA sorting will not work properly without being
able to specify a locale. Again, however, it is worth emphasizing the usefulness
of the CLDR root collation order on its own. When defining a `Collator`, you can
set the default options (see below), which indicate the use of the CLDR table
with the "shifted" strategy. I think this is a good starting point.

\[0\]: There is also a variant form, `collate_no_tiebreak`, which will return
`Ordering::Equal` for any two strings that produce the same UCA sort key. (The
normal version will fall back on byte-value comparison in such cases.)

## Example usage

```rust
use feruca::Collator;

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
    naive.sort_unstable();

    let mut collator = Collator::default();
    uca.sort_unstable_by(|a, b| collator.collate(a, b));

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

    // Empty line for clarity
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

## Conformance

The UCA conformance tests can be run with the command `cargo test --release`.
Please note that, as a result of this library's reliance on `bstr` for UTF-8
validation, any
[surrogate code points](https://en.wikipedia.org/wiki/Universal_Character_Set_characters#Surrogates)
found in input to the `collate` method will be converted to the standard
"replacement character," `U+FFFD`. Conformant implementations of the UCA are
explicitly allowed to follow this approach. It does mean, however, that a
handful of lines (out of hundreds of thousands) in the conformance tests need to
be skipped. If you look at the `conformance` function in the tests module,
you'll see that any line containing a surrogate code point is passed over.

## Bincode

The binary files included with feruca represent hash tables of Unicode data.
They are generated in a separate repository,
[feruca-mapper](https://github.com/theodore-s-beers/feruca-mapper), and
serialized using [bincode](https://docs.rs/bincode/). You can rebuild them
yourself, if you prefer.

## Licensing

The text files in the `test-data` directory are covered by the
[Unicode License Agreement](https://www.unicode.org/license.txt). Everything
else is MIT-licensed.
