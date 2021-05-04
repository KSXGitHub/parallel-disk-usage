use super::{ChildPosition, Direction, Parenthood};
use derive_more::{AsMut, AsRef, Deref, DerefMut, Display, From, Into};
use fmt_iter::FmtIter;
use pipe_trait::Pipe;
use std::fmt::{Display, Error, Formatter};
use zero_copy_pads::{ExcessHandler, PaddedValue, Width};

/// Determine 3 characters to use as skeletal component that connect a node
/// to the rest of the tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TreeSkeletalComponent {
    /// Whether the node is the last child amongst its parent's `children`.
    pub child_position: ChildPosition,
    /// The direction of the visualization of the tree.
    pub direction: Direction,
    /// Whether the node has children.
    pub parenthood: Parenthood,
}

/// String made by calling [`TreeSkeletalComponent::visualize`](TreeSkeletalComponent).
#[derive(Debug, Clone, Copy, PartialEq, Eq, AsRef, Deref, Display, Into)]
pub struct TreeSkeletalComponentVisualization(&'static str);

impl TreeSkeletalComponent {
    /// Determine 3 characters to use as skeletal component that connect a node
    /// to the rest of the tree.
    pub const fn visualize(self) -> TreeSkeletalComponentVisualization {
        use ChildPosition::*;
        use Direction::*;
        use Parenthood::*;
        let result = match (self.child_position, self.direction, self.parenthood) {
            (Init, BottomUp, Parent) => "├─┴",
            (Init, BottomUp, Childless) => "├──",
            (Init, TopDown, Parent) => "├─┬",
            (Init, TopDown, Childless) => "├──",
            (Last, BottomUp, Parent) => "┌─┴",
            (Last, BottomUp, Childless) => "┌──",
            (Last, TopDown, Parent) => "└─┬",
            (Last, TopDown, Childless) => "└──",
        };
        TreeSkeletalComponentVisualization(result)
    }
}

impl Display for TreeSkeletalComponent {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result<(), Error> {
        write!(formatter, "{}", self.visualize())
    }
}

impl Width for TreeSkeletalComponentVisualization {
    fn width(&self) -> usize {
        self.len()
    }
}

/// Horizontal slice of a tree of the height of exactly 1 line of text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeHorizontalSlice<Name: Width> {
    pub(super) ancestor_relative_positions: Vec<ChildPosition>,
    pub(super) skeleton: TreeSkeletalComponentVisualization,
    pub(super) name: Name,
}

impl<Name: Width> TreeHorizontalSlice<Name> {
    #[inline]
    fn depth(&self) -> usize {
        self.ancestor_relative_positions.len()
    }

    #[inline]
    fn indent_width(&self) -> usize {
        self.depth() * 2
    }

    #[inline]
    fn required_width(&self) -> usize {
        self.indent_width() + self.skeleton.width()
    }

    #[inline]
    fn indent(&self) -> impl Display + '_ {
        self.ancestor_relative_positions
            .iter()
            .map(|position| match position {
                ChildPosition::Init => "│ ",
                ChildPosition::Last => "  ",
            })
            .pipe(FmtIter::from)
    }

    #[inline]
    pub(super) fn resolve_padded_maybe<PadBlock, HandleExcess, Pad>(
        source: PaddedValue<MaybeTreeHorizontalSlice<Name>, PadBlock, HandleExcess, Pad>,
    ) -> Option<PaddedValue<Self, PadBlock, HandleExcess, Pad>>
    where
        PadBlock: Display,
        HandleExcess:
            ExcessHandler<MaybeTreeHorizontalSlice<Name>, PadBlock> + ExcessHandler<Self, PadBlock>,
        Pad: zero_copy_pads::Pad<MaybeTreeHorizontalSlice<Name>, PadBlock>
            + zero_copy_pads::Pad<Self, PadBlock>,
    {
        let PaddedValue {
            value,
            pad_block,
            total_width,
            pad,
            handle_excess,
        } = source;
        value.0.map(|value| PaddedValue {
            value,
            pad_block,
            total_width,
            pad,
            handle_excess,
        })
    }
}

impl<Name: Width> Display for TreeHorizontalSlice<Name> {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result<(), Error> {
        write!(
            formatter,
            "{}{}{}",
            self.indent(),
            self.skeleton,
            &self.name,
        )
    }
}

impl<Name: Width> Width for TreeHorizontalSlice<Name> {
    fn width(&self) -> usize {
        self.required_width() + self.name.width()
    }
}

impl TreeHorizontalSlice<String> {
    /// Truncate the name to fit specified `max_width`.
    ///
    /// * If `max_width` is already sufficient, do nothing other than return `Ok(())`.
    /// * If `max_width` is insufficient even for the required part, return `Err(N)`
    ///   where `N` is the required width.
    /// * If `max_width` is sufficient for the required part but insufficient for the
    ///   name, truncate and add `"..."` to the name.
    pub fn truncate(&mut self, max_width: usize) -> Result<(), usize> {
        if self.width() <= max_width {
            return Ok(());
        }

        let min_width = self.required_width() + "...".len();
        if min_width >= max_width {
            return Err(min_width);
        }

        self.name.truncate(max_width - min_width);
        self.name += "...";
        Ok(())
    }
}

/// [`Option`] of [`TreeHorizontalSlice`] that can be inserted into
/// [`PaddedColumnIter`](zero_copy_pads::PaddedColumnIter).
#[derive(Debug, Clone, PartialEq, Eq, AsRef, Deref, AsMut, DerefMut, From, Into)]
pub struct MaybeTreeHorizontalSlice<Name: Width>(Option<TreeHorizontalSlice<Name>>);

impl<Name: Width> Width for MaybeTreeHorizontalSlice<Name> {
    fn width(&self) -> usize {
        if let Some(content) = self.as_ref() {
            content.width()
        } else {
            0
        }
    }
}

impl<Name: Width> Display for MaybeTreeHorizontalSlice<Name> {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result<(), Error> {
        if let Some(content) = self.as_ref() {
            content.fmt(formatter)?;
        }

        Ok(())
    }
}
