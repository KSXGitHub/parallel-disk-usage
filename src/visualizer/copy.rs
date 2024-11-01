use super::Visualizer;
use crate::size;
use std::fmt::Display;

impl<'a, Name, Size> Clone for Visualizer<'a, Name, Size>
where
    Name: Display,
    Size: size::Size,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, Name, Size> Copy for Visualizer<'a, Name, Size>
where
    Name: Display,
    Size: size::Size,
{
}
