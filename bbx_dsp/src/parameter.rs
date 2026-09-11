//! Parameter modulation system.
//!
//! A [`Parameter`] is a base value plus up to [`MAX_MODULATION_ROUTES`]
//! [`ModulationRoute`]s. Each route names a [`ModulationSource`] (a block and one
//! of its modulation outputs) and a depth. The value read during processing is
//!
//! ```text
//! base + Σ depth_i · source_i
//! ```
//!
//! A parameter with no routes is a constant. Depth lives on the route, not in
//! the modulator, so one LFO can drive two targets at different amounts.
//!
//! [`ModulationValues`] is the read-only view of every modulator output the
//! graph collected for the current buffer. Blocks driven outside a graph pass
//! [`ModulationValues::empty`].

use core::fmt;

use bbx_core::StackVec;

use crate::{block::BlockId, sample::Sample};

/// Maximum number of modulation routes a single parameter can hold.
pub const MAX_MODULATION_ROUTES: usize = 4;

/// One modulation output of one block.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ModulationSource {
    /// The modulator block.
    pub block: BlockId,
    /// Index into that block's [`modulation_outputs`](crate::block::Block::modulation_outputs).
    pub output: usize,
}

impl ModulationSource {
    /// Create a source from a block and one of its modulation output indices.
    #[inline]
    pub const fn new(block: BlockId, output: usize) -> Self {
        Self { block, output }
    }

    /// Create a source for a block's first modulation output.
    #[inline]
    pub const fn first_output(block: BlockId) -> Self {
        Self { block, output: 0 }
    }
}

/// A single modulation connection into a [`Parameter`].
///
/// Fields are private and set through [`ModulationRoute::new`] so that later
/// additions (such as a per-route rate) are not breaking changes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ModulationRoute<S: Sample> {
    source: ModulationSource,
    depth: S,
}

impl<S: Sample> ModulationRoute<S> {
    /// Create a route from `source` scaled by `depth`.
    #[inline]
    pub const fn new(source: ModulationSource, depth: S) -> Self {
        Self { source, depth }
    }

    /// Create a route from `source` with unity depth.
    #[inline]
    pub fn unity(source: ModulationSource) -> Self {
        Self { source, depth: S::ONE }
    }

    /// The modulation source this route reads.
    #[inline]
    pub const fn source(&self) -> ModulationSource {
        self.source
    }

    /// The scale applied to the source value.
    #[inline]
    pub const fn depth(&self) -> S {
        self.depth
    }

    /// Change the scale applied to the source value.
    #[inline]
    pub fn set_depth(&mut self, depth: S) {
        self.depth = depth;
    }
}

/// Error returned when a [`Parameter`] cannot accept another route.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParameterError {
    /// The parameter already holds [`MAX_MODULATION_ROUTES`] routes.
    RoutesFull {
        /// The capacity that was exceeded.
        capacity: usize,
    },
}

impl fmt::Display for ParameterError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RoutesFull { capacity } => {
                write!(
                    formatter,
                    "parameter already holds the maximum of {capacity} modulation routes"
                )
            }
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for ParameterError {}

/// Read-only view of the modulation values collected by a graph for one buffer.
///
/// `values` is flat; `offsets` has one entry per block plus a terminator, so
/// block `b`'s outputs occupy `values[offsets[b]..offsets[b + 1]]`. Lookups that
/// fall outside either range read as zero rather than panicking, because this
/// runs on the audio thread.
#[derive(Debug, Clone, Copy)]
pub struct ModulationValues<'a, S: Sample> {
    values: &'a [S],
    offsets: &'a [usize],
}

impl<'a, S: Sample> ModulationValues<'a, S> {
    /// Wrap a flat value slice and its per-block offset table.
    #[inline]
    pub const fn new(values: &'a [S], offsets: &'a [usize]) -> Self {
        Self { values, offsets }
    }

    /// A view with no modulators; every lookup reads as zero.
    #[inline]
    pub const fn empty() -> Self {
        Self {
            values: &[],
            offsets: &[],
        }
    }

    /// The current value of `source`, or zero if the source does not exist.
    #[inline]
    pub fn get(&self, source: ModulationSource) -> S {
        let block = source.block.0;
        match (self.offsets.get(block), self.offsets.get(block + 1)) {
            (Some(&start), Some(&end)) => {
                let index = start + source.output;
                if index < end {
                    self.values.get(index).copied().unwrap_or(S::ZERO)
                } else {
                    S::ZERO
                }
            }
            _ => S::ZERO,
        }
    }
}

impl<S: Sample> Default for ModulationValues<'_, S> {
    fn default() -> Self {
        Self::empty()
    }
}

/// A block parameter: a base value plus summed modulation routes.
///
/// Construct constants with [`Parameter::constant`] or `value.into()`; wire
/// modulation through [`GraphBuilder::modulate`](crate::graph::GraphBuilder::modulate),
/// which calls [`Parameter::add_route`] on the named parameter.
#[derive(Debug, Clone)]
pub struct Parameter<S: Sample> {
    base: S,
    routes: StackVec<ModulationRoute<S>, MAX_MODULATION_ROUTES>,
}

impl<S: Sample> Parameter<S> {
    /// A parameter with no modulation.
    #[inline]
    pub const fn constant(base: S) -> Self {
        Self {
            base,
            routes: StackVec::new(),
        }
    }

    /// The unmodulated value.
    #[inline]
    pub const fn base(&self) -> S {
        self.base
    }

    /// Replace the unmodulated value, keeping the routes.
    #[inline]
    pub fn set_base(&mut self, base: S) {
        self.base = base;
    }

    /// The modulation routes feeding this parameter.
    #[inline]
    pub fn routes(&self) -> &[ModulationRoute<S>] {
        self.routes.as_slice()
    }

    /// Mutable access to the routes, for adjusting depth after wiring.
    #[inline]
    pub fn routes_mut(&mut self) -> &mut [ModulationRoute<S>] {
        self.routes.as_mut_slice()
    }

    /// `true` when at least one route is attached.
    #[inline]
    pub fn has_routes(&self) -> bool {
        !self.routes.is_empty()
    }

    /// Attach a route. Fails when the parameter already holds
    /// [`MAX_MODULATION_ROUTES`] routes.
    pub fn add_route(&mut self, route: ModulationRoute<S>) -> Result<(), ParameterError> {
        self.routes.push(route).map_err(|_| ParameterError::RoutesFull {
            capacity: MAX_MODULATION_ROUTES,
        })
    }

    /// Detach every route, leaving the base value.
    #[inline]
    pub fn clear_routes(&mut self) {
        self.routes.clear();
    }

    /// The modulated value for the current buffer: `base + Σ depth · source`.
    #[inline]
    pub fn value(&self, modulation_values: &ModulationValues<S>) -> S {
        self.value_with_base(self.base, modulation_values)
    }

    /// The modulated value with `base` substituted for the stored base.
    ///
    /// Blocks whose base is decided elsewhere (an oscillator following a MIDI
    /// note) use this so routes still add on top of the substituted value.
    #[inline]
    pub fn value_with_base(&self, base: S, modulation_values: &ModulationValues<S>) -> S {
        let mut value = base;
        for route in self.routes.as_slice() {
            value += route.depth * modulation_values.get(route.source);
        }
        value
    }
}

impl<S: Sample> From<S> for Parameter<S> {
    fn from(base: S) -> Self {
        Self::constant(base)
    }
}

/// Describes a modulation output provided by a modulator block.
///
/// Modulator blocks (LFOs, envelopes) declare their outputs using this type,
/// specifying the output name and expected value range.
#[derive(Debug, Clone)]
pub struct ModulationOutput {
    /// Human-readable name for this output (e.g., "amplitude", "frequency").
    pub name: &'static str,

    /// Minimum value this output can produce.
    pub min_value: f64,

    /// Maximum value this output can produce.
    pub max_value: f64,
}

/// `true` when `name` matches any candidate, ignoring ASCII case.
///
/// Blocks use this to accept aliases such as `"level"` for `"level_db"`.
#[inline]
pub fn parameter_name_matches(name: &str, candidates: &[&str]) -> bool {
    candidates.iter().any(|candidate| candidate.eq_ignore_ascii_case(name))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Two blocks: block 0 has two outputs (10, 20), block 1 has one output (30).
    fn two_block_values() -> (Vec<f32>, Vec<usize>) {
        (vec![10.0, 20.0, 30.0], vec![0, 2, 3])
    }

    #[test]
    fn constant_ignores_modulation_values() {
        let parameter = Parameter::constant(42.0_f32);
        let (values, offsets) = two_block_values();
        assert_eq!(parameter.value(&ModulationValues::new(&values, &offsets)), 42.0);
        assert!(!parameter.has_routes());
    }

    #[test]
    fn constant_with_empty_view_f64() {
        let parameter: Parameter<f64> = 42.0.into();
        assert_eq!(parameter.value(&ModulationValues::empty()), 42.0);
    }

    #[test]
    fn single_route_unity_depth_adds_source() {
        let mut parameter = Parameter::constant(100.0_f32);
        parameter
            .add_route(ModulationRoute::unity(ModulationSource::first_output(BlockId(1))))
            .unwrap();
        let (values, offsets) = two_block_values();
        assert_eq!(parameter.value(&ModulationValues::new(&values, &offsets)), 130.0);
        assert!(parameter.has_routes());
    }

    #[test]
    fn depth_scales_source() {
        let mut parameter = Parameter::constant(0.0_f32);
        parameter
            .add_route(ModulationRoute::new(ModulationSource::new(BlockId(0), 1), 0.5))
            .unwrap();
        let (values, offsets) = two_block_values();
        assert_eq!(parameter.value(&ModulationValues::new(&values, &offsets)), 10.0);
    }

    #[test]
    fn two_routes_sum() {
        let mut parameter = Parameter::constant(1.0_f32);
        parameter
            .add_route(ModulationRoute::new(ModulationSource::new(BlockId(0), 0), 1.0))
            .unwrap();
        parameter
            .add_route(ModulationRoute::new(ModulationSource::new(BlockId(1), 0), 2.0))
            .unwrap();
        let (values, offsets) = two_block_values();
        assert_eq!(
            parameter.value(&ModulationValues::new(&values, &offsets)),
            1.0 + 10.0 + 60.0
        );
    }

    #[test]
    fn zero_depth_has_no_effect() {
        let mut parameter = Parameter::constant(5.0_f32);
        parameter
            .add_route(ModulationRoute::new(ModulationSource::new(BlockId(1), 0), 0.0))
            .unwrap();
        let (values, offsets) = two_block_values();
        assert_eq!(parameter.value(&ModulationValues::new(&values, &offsets)), 5.0);
    }

    #[test]
    fn second_output_is_addressable() {
        let (values, offsets) = two_block_values();
        let view = ModulationValues::new(&values, &offsets);
        assert_eq!(view.get(ModulationSource::new(BlockId(0), 0)), 10.0);
        assert_eq!(view.get(ModulationSource::new(BlockId(0), 1)), 20.0);
    }

    #[test]
    fn output_beyond_block_range_reads_zero() {
        let (values, offsets) = two_block_values();
        let view = ModulationValues::new(&values, &offsets);
        assert_eq!(view.get(ModulationSource::new(BlockId(1), 1)), 0.0);
        assert_eq!(view.get(ModulationSource::new(BlockId(0), 2)), 0.0);
    }

    #[test]
    fn unknown_block_reads_zero() {
        let (values, offsets) = two_block_values();
        let view = ModulationValues::new(&values, &offsets);
        assert_eq!(view.get(ModulationSource::new(BlockId(7), 0)), 0.0);
        assert_eq!(
            ModulationValues::<f32>::empty().get(ModulationSource::first_output(BlockId(0))),
            0.0
        );
    }

    #[test]
    fn routes_are_capped() {
        let mut parameter = Parameter::constant(0.0_f32);
        for _ in 0..MAX_MODULATION_ROUTES {
            parameter
                .add_route(ModulationRoute::unity(ModulationSource::first_output(BlockId(0))))
                .unwrap();
        }
        let overflow = parameter.add_route(ModulationRoute::unity(ModulationSource::first_output(BlockId(0))));
        assert_eq!(
            overflow,
            Err(ParameterError::RoutesFull {
                capacity: MAX_MODULATION_ROUTES
            })
        );
        assert_eq!(parameter.routes().len(), MAX_MODULATION_ROUTES);
    }

    #[test]
    fn value_with_base_substitutes_base_and_keeps_routes() {
        let mut parameter = Parameter::constant(440.0_f32);
        parameter
            .add_route(ModulationRoute::unity(ModulationSource::first_output(BlockId(1))))
            .unwrap();
        let (values, offsets) = two_block_values();
        let view = ModulationValues::new(&values, &offsets);
        assert_eq!(parameter.value_with_base(880.0, &view), 910.0);
    }

    #[test]
    fn set_base_and_clear_routes() {
        let mut parameter = Parameter::constant(1.0_f32);
        parameter
            .add_route(ModulationRoute::unity(ModulationSource::first_output(BlockId(1))))
            .unwrap();
        parameter.set_base(2.0);
        parameter.clear_routes();
        assert_eq!(parameter.base(), 2.0);
        assert!(!parameter.has_routes());
    }

    #[test]
    fn routes_mut_adjusts_depth() {
        let mut parameter = Parameter::constant(0.0_f32);
        parameter
            .add_route(ModulationRoute::unity(ModulationSource::first_output(BlockId(1))))
            .unwrap();
        parameter.routes_mut()[0].set_depth(0.1);
        let (values, offsets) = two_block_values();
        let value = parameter.value(&ModulationValues::new(&values, &offsets));
        assert!((value - 3.0).abs() < 1e-6);
    }

    #[test]
    fn clone_preserves_routes() {
        let mut parameter = Parameter::constant(1.0_f64);
        parameter
            .add_route(ModulationRoute::new(ModulationSource::first_output(BlockId(1)), 2.0))
            .unwrap();
        let clone = parameter.clone();
        assert_eq!(clone.base(), 1.0);
        assert_eq!(clone.routes(), parameter.routes());
    }

    #[test]
    fn name_matching_ignores_case_and_accepts_aliases() {
        assert!(parameter_name_matches("Frequency", &["frequency"]));
        assert!(parameter_name_matches("q", &["resonance", "q"]));
        assert!(!parameter_name_matches("cutoff", &["frequency"]));
    }

    #[test]
    fn modulation_output_creation() {
        let output = ModulationOutput {
            name: "test",
            min_value: -1.0,
            max_value: 1.0,
        };
        assert_eq!(output.name, "test");
        assert!((output.min_value - (-1.0)).abs() < 1e-10);
        assert!((output.max_value - 1.0).abs() < 1e-10);
    }
}
