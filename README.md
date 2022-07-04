# feruca – the Unicode Collation Algorithm in Rust

feruca is a basic implementation of the [Unicode Collation Algorithm](https://unicode.org/reports/tr10/) in 100% safe Rust. (One unsafe standard library function, `char::from_u32_unchecked`, is used for testing—but only for testing!) The name of the library is a portmanteau of Ferris 🦀 and UCA.

I mean a few things by "basic implementation." First, I don't expect that this is currently very performant. My focus has been on passing the official [conformance tests](https://www.unicode.org/Public/UCA/latest/CollationTest.html). (feruca also passes the conformance tests for the [CLDR](https://github.com/unicode-org/cldr) root collation order; more on this below.) Second, there is not yet support for tailoring, beyond being able to choose between the Default Unicode Collation Element Table (DUCET) and the default variation from CLDR. (You can also choose between the "non-ignorable" and "shifted" strategies for handling variable-weight characters.) Adding further support for tailoring is a near-term priority. Third, the library has at present only one public function: `collate`, which accepts two string references (plus a `CollationOptions` struct), and returns an `Ordering`. That is, you can pass `collate` to the standard library function `sort_by` (see the example below).

For many people and use cases, UCA sorting will not work properly without being able to specify a certain locale. That being said, the CLDR root collation order is already quite useful. When calling the `collate` function, you can pass default options (see below), which specify the use of the CLDR table with the "shifted" strategy. I think this is a good starting point.

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

    uca.sort_by(|a, b| collate(a, b, &CollationOptions::default()));
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
