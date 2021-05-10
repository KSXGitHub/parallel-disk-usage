use derive_more::{AsMut, AsRef, Deref, DerefMut, From, FromStr};
use std::{
    ffi::{OsStr, OsString},
    fmt::{Debug, Display, Error, Formatter},
};

/// [`Display`] inner [`OsStr`] or [`OsString`].
///
/// If the inner string can be converted to UTF-8, displays the UTF-8.
/// Otherwise, displays its [`Debug`] form.
#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    AsMut,
    AsRef,
    Deref,
    DerefMut,
    From,
    FromStr,
)]
pub struct OsStringDisplay<Inner = OsString>(pub Inner)
where
    Inner: AsRef<OsStr> + Debug;

impl<Inner> OsStringDisplay<Inner>
where
    Inner: AsRef<OsStr> + Debug,
{
    /// Get immutable reference to the inner value.
    #[inline]
    pub fn inner(&self) -> &Inner {
        &self.0
    }

    /// Get immutable reference to the inner `OsStr`.
    #[inline]
    pub fn as_os_str(&self) -> &OsStr {
        self.inner().as_ref()
    }
}

impl<Inner> AsRef<OsStr> for OsStringDisplay<Inner>
where
    Inner: AsRef<OsStr> + Debug,
{
    fn as_ref(&self) -> &OsStr {
        self.as_os_str()
    }
}

impl<Inner> Display for OsStringDisplay<Inner>
where
    Inner: AsRef<OsStr> + Debug,
{
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result<(), Error> {
        let inner = self.as_os_str();
        if let Some(utf8) = inner.to_str() {
            write!(formatter, "{}", utf8)
        } else {
            write!(formatter, "{:?}", inner)
        }
    }
}

#[cfg(test)]
#[cfg(unix)]
mod test;
