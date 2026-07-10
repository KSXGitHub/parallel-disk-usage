/// The production provider for the crate's dependency-injection seams.
///
/// It implements every capability trait defined for testing (see the
/// "Dependency Injection for Tests" section of `CONTRIBUTING.md`) by
/// delegating to the real operating system. Production call sites name it
/// explicitly through a turbofish, such as `any_path_is_in_hdd::<Host>(…)`,
/// so the production choice of provider is visible at the call site.
///
/// The capability traits themselves, together with their implementations for
/// this provider, live next to the logic that consumes them: the disk and
/// filesystem capabilities in [`hdd`](super::hdd), and the argument
/// resolution capabilities in
/// [`overlapping_arguments`](super::overlapping_arguments).
pub struct Host;
