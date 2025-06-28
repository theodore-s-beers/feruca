# feruca – Unicode collation in Rust

feruca is a simple, from-scratch implementation of the
[Unicode Collation Algorithm](https://unicode.org/reports/tr10/) in Rust. It's
current with **Unicode v16** (and, correspondingly, **CLDR v46.1**; see below).
The name of the library is a portmanteau of Ferris 🦀 and UCA.

No `unsafe` is used directly in this library: `#![forbid(unsafe_code)]`. It
relies on the well-vetted [bstr](https://github.com/BurntSushi/bstr) to accept
input (in the form of either `&str` or `&[u8]`), to perform UTF-8 validation,
and to generate a list of Unicode scalar values, which can then be processed for
collation. The idea is to be tolerant of input that may not be entirely kosher
UTF-8.

In describing feruca as a "simple implementation," I have a few things in mind.
**First**, the performance of the library could perhaps still be improved—at
least, in comparison to the official C implementation, `ucol` from
[icu4c](https://github.com/unicode-org/icu), which is incredibly optimized. I no
longer run benchmarks against that implementation, but feruca was always slower,
and my guess is that it still is (though not severely). What I _do_ currently
[benchmark](https://github.com/theodore-s-beers/feruca-benchmarks) against is
the newer first-party implementation belonging to the
[icu4x](https://github.com/unicode-org/icu4x) project, which is also written in
Rust. feruca performs **on the order of 2–4x faster** than the icu4x
collator—while having a much smaller feature set. My priority as a solo dev was
to produce a relatively bare-bones implementation that passes the official UCA
[conformance tests](https://www.unicode.org/Public/UCA/latest/CollationTest.html),
as well as the tests for the "root collation order" of the
[Common Locale Data Repository](https://github.com/unicode-org/cldr) (CLDR).

**Second**, support for tailoring is minimal (so far). You can choose between
two tables of character weights: the Default Unicode Collation Element Table
(DUCET), or the CLDR variant thereof. The CLDR table then becomes the starting
point for actual collation tailoring based on language/locale. I have added only
two tailorings, both intended for use with Arabic-script languages. One of them
shifts letters in the Arabic script so that, as a block, they sort before the
Latin script. The other tailoring attempts to interleave the Latin and Arabic
scripts, so that _alif_ sorts after A and before B; _bā’_ sorts after B and
before C; etc. This is enough for my own work with Persian and Arabic texts. The
CLDR table in its unmodified form—i.e., the root collation order—works
out-of-the-box for several other languages. I do hope to add more tailorings,
but it will be a gradual process, and driven by demand. Realistically, feruca
will never have the kind of all-encompassing, flexible support for tailoring
that is provided by ICU. My feeling is that there is a place for less
sophisticated solutions, with simpler APIs, smaller dependency trees, etc. (If
you have thoughts on this, I would be interested in hearing them.)

Apart from locale tailoring, you can choose between the "non-ignorable" and
"shifted" strategies for handling variable-weight characters—with the latter
being the default. There is also an option to use byte-value comparison as a
"tiebreaker" in cases where two strings produce identical UCA sort keys.

**Third**, this library has effectively just one public method, `collate`,
belonging to a struct, `Collator`, which sets the options. `collate` accepts two
string references or byte slices, and returns an `Ordering` value. It is
designed to be passed as a comparator to the standard library method `sort_by`
(or `sort_unstable_by`). See "Example usage" below.

For many people and use cases, UCA sorting will not work properly without being
able to specify a locale! Again, however, it is worth emphasizing the usefulness
of the CLDR root collation order on its own. When defining a `Collator`, you can
set the default options (see below), which indicate the use of the CLDR table
with the "shifted" strategy. I think this is a good starting point.

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
        println!("{item}");
    }
    // Éloi
    // Elrond
    // Melissa
    // Mélissa
    // Ötzi
    // Overton
    // چنگیز
    // صدام

    println!(); // Empty line for clarity

    for item in naive {
        println!("{item}");
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
be skipped. If you look at the `conformance` function in the tests module, you
will see that any line containing a surrogate code point is passed over.

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
