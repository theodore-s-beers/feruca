use crate::ascii::{AsciiResult, compare_ascii_primary_non_ignorable, fill_and_check};
use crate::cea::{LazyPrimaryResult, compare_primary_streaming, compare_primary_streaming_utf8};
use crate::consts::{CLDR_ROOT, DUCET, LOW_CLDR, LOW_DUCET};
use crate::first_weight::try_initial;
use crate::normalize::make_nfd;
use crate::prefix::{find_byte_prefix, find_prefix_shifted};
use crate::sort_key::compare_incremental;
use crate::tables::CollationTable;
use crate::tailor::{ARABIC_INTERLEAVED, ARABIC_SCRIPT};
use crate::{Locale, Tailoring};
use bstr::{B, ByteSlice};
use std::cmp::Ordering;

const USE_LAZY_UTF8_PRIMARY: bool = true;
const LAZY_UTF8_PRIMARY_MIN_COMBINED_BYTES: usize = 64;

#[cfg(feature = "pipeline-stats")]
/// Diagnostic counters for `Collator::collate`.
///
/// These counters are intended for benchmarking and pipeline experiments. They are available only
/// when the `pipeline-stats` feature is enabled.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PipelineStats {
    /// Total calls to `Collator::collate`.
    pub comparisons: u64,
    /// Calls that returned from the initial exact-equality check.
    pub equal_early: u64,
    /// Calls where a byte prefix was trimmed before UTF-8 decoding.
    pub byte_prefix_trimmed: u64,
    /// Total bytes trimmed by byte-prefix trimming, counted once per compared pair.
    pub byte_prefix_bytes_trimmed: u64,
    /// Calls resolved by the non-ignorable ASCII primary fast path.
    pub ascii_primary_resolved: u64,
    /// Calls that attempted lazy UTF-8 primary streaming before filling code point buffers.
    pub lazy_utf8_primary_attempts: u64,
    /// Calls resolved by lazy UTF-8 primary streaming before filling code point buffers.
    pub lazy_utf8_primary_resolved: u64,
    /// Calls that reused a lazy UTF-8 CE prefix and materialized only the remaining suffix.
    pub lazy_utf8_prefix_reused: u64,
    /// Calls where lazy UTF-8 primary streaming had to discard its work and use full fallback.
    pub lazy_utf8_full_fallback: u64,
    /// Calls resolved while filling code point buffers with ASCII-aware comparison.
    pub fill_ascii_resolved: u64,
    /// Input sides normalized to NFD.
    pub nfd_normalizations: u64,
    /// Total code points decoded into the comparison buffers before NFD normalization.
    pub codepoints_decoded: u64,
    /// Calls where a code point prefix was trimmed after UTF-8 decoding and normalization.
    pub codepoint_prefix_trimmed: u64,
    /// Total code points trimmed by code point prefix trimming, counted once per compared pair.
    pub codepoint_prefix_codepoints_trimmed: u64,
    /// Calls resolved by the initial-primary check.
    pub initial_primary_resolved: u64,
    /// Calls resolved by streaming primary collation element generation.
    pub streaming_primary_resolved: u64,
    /// Total code points consumed by streaming primary generation after code point prefix trimming.
    pub codepoints_consumed_primary: u64,
    /// Calls that reached secondary/tertiary/quaternary comparison.
    pub later_levels_reached: u64,
    /// Calls resolved by secondary/tertiary/quaternary comparison.
    pub later_levels_resolved: u64,
    /// Calls resolved by byte-value tiebreaking after equivalent collation weights.
    pub tiebreak_resolved: u64,
}

/// The `Collator` struct is the entry point for this library's API. It defines the options to be
/// used in collation. The method `collate` will then compare two string references (or byte slices)
/// according to the selected options, and return an `Ordering` value.
///
/// You can choose between two tables of character weights: DUCET and CLDR. With the CLDR table,
/// there is a further choice of locale tailoring. The `Root` locale represents the table in its
/// unmodified form. The `ArabicScript` locale shifts the weights of Arabic-script letters so that
/// they sort before the Latin script; and the `ArabicInterleaved` locale mixes the two scripts, so
/// that, e.g., _alif_ sorts between A and B, and _bā’_ between B and C. Further locales will be
/// added over time.
///
/// You can also choose between two approaches to the handling of variable-weight characters:
/// "non-ignorable" and "shifted." Finally, you can select whether to use byte-value comparison as a
/// tiebreaker when two strings produce identical Unicode Collation Algorithm sort keys.
///
/// The default for `Collator` is to use the CLDR table with the `Root` locale; to use the "shifted"
/// approach for variable-weight characters; and to break ties with byte-value comparison. This
/// should be a good starting point for collation in many languages.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash)]
pub struct Collator {
    /// The table of weights to be used: DUCET or CLDR (with a choice of locale for the latter)
    pub tailoring: Tailoring,
    /// The approach to handling variable-weight characters: "non-ignorable" (i.e., `false`) or
    /// "shifted" (i.e., `true`)
    pub shifting: bool,
    /// Whether to use byte-value comparison as a tiebreaker when two strings produce identical
    /// Unicode Collation Algorithm sort keys
    pub tiebreak: bool,
    a_chars: Vec<u32>,
    b_chars: Vec<u32>,
    a_cea: Vec<u32>,
    b_cea: Vec<u32>,
    #[cfg(feature = "pipeline-stats")]
    stats: PipelineStats,
}

impl Default for Collator {
    fn default() -> Self {
        Self::new(Tailoring::default(), true, true)
    }
}

impl Collator {
    /// Create a new `Collator` with the specified options. NB: it is also possible to call
    /// `Collator::default()`.
    #[must_use]
    pub fn new(tailoring: Tailoring, shifting: bool, tiebreak: bool) -> Self {
        Self {
            tailoring,
            shifting,
            tiebreak,
            a_chars: Vec::new(),
            b_chars: Vec::new(),
            a_cea: vec![0; 64],
            b_cea: vec![0; 64],

            #[cfg(feature = "pipeline-stats")]
            stats: PipelineStats::default(),
        }
    }

    /// Return diagnostic counters for this collator.
    ///
    /// This method is available only when the `pipeline-stats` feature is enabled.
    #[cfg(feature = "pipeline-stats")]
    #[must_use]
    pub const fn stats(&self) -> &PipelineStats {
        &self.stats
    }

    /// Reset diagnostic counters for this collator.
    ///
    /// This method is available only when the `pipeline-stats` feature is enabled.
    #[cfg(feature = "pipeline-stats")]
    pub const fn clear_stats(&mut self) {
        self.stats = PipelineStats {
            comparisons: 0,
            equal_early: 0,
            byte_prefix_trimmed: 0,
            byte_prefix_bytes_trimmed: 0,
            ascii_primary_resolved: 0,
            lazy_utf8_primary_attempts: 0,
            lazy_utf8_primary_resolved: 0,
            lazy_utf8_prefix_reused: 0,
            lazy_utf8_full_fallback: 0,
            fill_ascii_resolved: 0,
            nfd_normalizations: 0,
            codepoints_decoded: 0,
            codepoint_prefix_trimmed: 0,
            codepoint_prefix_codepoints_trimmed: 0,
            initial_primary_resolved: 0,
            streaming_primary_resolved: 0,
            codepoints_consumed_primary: 0,
            later_levels_reached: 0,
            later_levels_resolved: 0,
            tiebreak_resolved: 0,
        };
    }

    /// This is the primary method in the library. It accepts as arguments two string references or
    /// byte slices; compares them using the options chosen; and returns an `Ordering` value. This
    /// is designed to be passed to the `sort_by` (or `sort_unstable_by`) function in the standard
    /// library. Simple usage might look like the following...
    ///
    /// ```
    /// use feruca::Collator;
    ///
    /// let mut collator = Collator::default();
    ///
    /// let mut names = ["Peng", "Peña", "Ernie", "Émile"];
    /// names.sort_unstable_by(|a, b| collator.collate(a, b));
    ///
    /// let expected = ["Émile", "Ernie", "Peña", "Peng"];
    /// assert_eq!(names, expected);
    /// ```
    #[allow(clippy::too_many_lines)]
    pub fn collate<T: AsRef<[u8]> + Ord + ?Sized>(&mut self, a: &T, b: &T) -> Ordering {
        #[cfg(feature = "pipeline-stats")]
        {
            self.stats.comparisons += 1;
        }

        // Early out; equal is equal
        if a == b {
            #[cfg(feature = "pipeline-stats")]
            {
                self.stats.equal_early += 1;
            }

            return Ordering::Equal;
        }

        let a_bytes = a.as_ref();
        let b_bytes = b.as_ref();
        let mut ctx = None;

        let byte_offset = if has_byte_prefix(a_bytes, b_bytes) {
            let current_ctx =
                ctx.get_or_insert_with(|| CollationContext::new(self.shifting, self.tailoring));
            find_byte_prefix(a_bytes, b_bytes, current_ctx)
        } else {
            0
        };

        #[cfg(feature = "pipeline-stats")]
        if byte_offset > 0 {
            self.stats.byte_prefix_trimmed += 1;
            self.stats.byte_prefix_bytes_trimmed += u64::try_from(byte_offset).unwrap_or(u64::MAX);
        }

        let a_bytes = &a_bytes[byte_offset..];
        let b_bytes = &b_bytes[byte_offset..];

        if !self.shifting {
            let current_ctx =
                ctx.get_or_insert_with(|| CollationContext::new(self.shifting, self.tailoring));
            if let Some(comparison) =
                compare_ascii_primary_non_ignorable(a_bytes, b_bytes, current_ctx.low)
            {
                #[cfg(feature = "pipeline-stats")]
                {
                    self.stats.ascii_primary_resolved += 1;
                }

                return comparison;
            }
        }

        if USE_LAZY_UTF8_PRIMARY
            && a_bytes.len() + b_bytes.len() >= LAZY_UTF8_PRIMARY_MIN_COMBINED_BYTES
        {
            let current_ctx =
                ctx.get_or_insert_with(|| CollationContext::new(self.shifting, self.tailoring));
            #[cfg(feature = "pipeline-stats")]
            {
                self.stats.lazy_utf8_primary_attempts += 1;
            }
            match compare_primary_streaming_utf8(
                &mut self.a_cea,
                &mut self.b_cea,
                a_bytes,
                b_bytes,
                current_ctx,
            ) {
                LazyPrimaryResult::Decided(comparison) => {
                    #[cfg(feature = "pipeline-stats")]
                    {
                        self.stats.lazy_utf8_primary_resolved += 1;
                    }

                    return comparison;
                }
                LazyPrimaryResult::ReusablePrefix | LazyPrimaryResult::NeedsFullFallback => {
                    #[cfg(feature = "pipeline-stats")]
                    {
                        self.stats.lazy_utf8_full_fallback += 1;
                    }
                }
            }
        }

        // Validate UTF-8 and make iterators for u32 code points
        let mut a_iter = B(a_bytes).chars().map(|c| c as u32);
        let mut b_iter = B(b_bytes).chars().map(|c| c as u32);

        // Clear code point Vecs
        self.a_chars.clear();
        self.b_chars.clear();

        // While iterating through input strings and filling code point Vecs, try to get a result by
        // comparing ASCII characters. This can avoid a lot of computation.
        let ascii_result = fill_and_check(
            &mut a_iter,
            &mut b_iter,
            &mut self.a_chars,
            &mut self.b_chars,
        );

        #[cfg(feature = "pipeline-stats")]
        {
            self.stats.codepoints_decoded +=
                u64::try_from(self.a_chars.len() + self.b_chars.len()).unwrap_or(u64::MAX);
        }

        let (a_needs_nfd, b_needs_nfd) = match ascii_result {
            AsciiResult::Done(o) => {
                #[cfg(feature = "pipeline-stats")]
                {
                    self.stats.fill_ascii_resolved += 1;
                }

                return o;
            }
            AsciiResult::Continue {
                a_needs_nfd,
                b_needs_nfd,
            } => (a_needs_nfd, b_needs_nfd),
        };

        // Normalize to NFD if necessary
        if a_needs_nfd {
            #[cfg(feature = "pipeline-stats")]
            {
                self.stats.nfd_normalizations += 1;
            }

            make_nfd(&mut self.a_chars);
        }
        if b_needs_nfd {
            #[cfg(feature = "pipeline-stats")]
            {
                self.stats.nfd_normalizations += 1;
            }

            make_nfd(&mut self.b_chars);
        }

        // Define collation context for subsequent steps
        let ctx = ctx.get_or_insert_with(|| CollationContext::new(self.shifting, self.tailoring));

        // In shifted mode, trimming a shared code point prefix can avoid carrying variable-weight
        // CEs into later levels. In non-ignorable mode, earlier byte/ASCII/lazy-primary paths have
        // usually already done enough prefix work that this pass is just overhead.
        let offset = if self.shifting {
            find_prefix_shifted(&self.a_chars, &self.b_chars, ctx)
        } else {
            0
        };

        #[cfg(feature = "pipeline-stats")]
        if offset > 0 {
            self.stats.codepoint_prefix_trimmed += 1;
            self.stats.codepoint_prefix_codepoints_trimmed +=
                u64::try_from(offset).unwrap_or(u64::MAX);
        }

        // Prefix trimming may reveal that one Vec is a prefix of the other
        if self.a_chars[offset..].is_empty() || self.b_chars[offset..].is_empty() {
            return self.a_chars.len().cmp(&self.b_chars.len());
        }

        // One last early out: if the opening code points of the Vecs are different, and neither
        // requires checking for a multi-code-point sequence, then we can try comparing their first
        // primary weights. If those are different, and both non-zero, it's decisive.
        if let Some(o) = try_initial(ctx, &self.a_chars[offset..], &self.b_chars[offset..]) {
            #[cfg(feature = "pipeline-stats")]
            {
                self.stats.initial_primary_resolved += 1;
            }

            return o;
        }

        // Otherwise, compare primary weights while generating collation elements. If primary
        // weights tie, the generated buffers are complete and can be reused for later levels.
        if let Some(comparison) = compare_primary_streaming(
            &mut self.a_cea,
            &mut self.b_cea,
            &mut self.a_chars,
            &mut self.b_chars,
            ctx,
            offset,
            #[cfg(feature = "pipeline-stats")]
            &mut self.stats,
        ) {
            #[cfg(feature = "pipeline-stats")]
            {
                self.stats.streaming_primary_resolved += 1;
            }

            return comparison;
        }

        // Sort keys are processed incrementally, until they yield a result
        #[cfg(feature = "pipeline-stats")]
        {
            self.stats.later_levels_reached += 1;
        }

        let comparison = compare_incremental(&self.a_cea, &self.b_cea, ctx.shifting);

        if comparison == Ordering::Equal && self.tiebreak {
            #[cfg(feature = "pipeline-stats")]
            {
                self.stats.tiebreak_resolved += 1;
            }

            return a.cmp(b);
        }

        #[cfg(feature = "pipeline-stats")]
        if comparison != Ordering::Equal {
            self.stats.later_levels_resolved += 1;
        }

        comparison
    }
}

fn has_byte_prefix(a: &[u8], b: &[u8]) -> bool {
    a.first().zip(b.first()).is_some_and(|(x, y)| x == y)
}

pub struct CollationContext {
    pub shifting: bool,
    pub cldr: bool,
    pub table: &'static CollationTable,
    pub low: &'static [u32],
}

impl CollationContext {
    fn new(shifting: bool, tailoring: Tailoring) -> Self {
        let cldr = tailoring != Tailoring::Ducet;

        Self {
            shifting,
            cldr,
            table: get_collation_table(tailoring),
            low: if cldr { &LOW_CLDR } else { &LOW_DUCET },
        }
    }
}

fn get_collation_table(tailoring: Tailoring) -> &'static CollationTable {
    match tailoring {
        Tailoring::Cldr(Locale::ArabicScript) => &ARABIC_SCRIPT,
        Tailoring::Cldr(Locale::ArabicInterleaved) => &ARABIC_INTERLEAVED,
        Tailoring::Cldr(Locale::Root) => &CLDR_ROOT,
        Tailoring::Ducet => &DUCET,
    }
}
