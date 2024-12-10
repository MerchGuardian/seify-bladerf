use std::{cmp, ffi::CStr};

use libbladerf_sys::bladerf_version;

#[derive(Copy, Clone, Debug)]
pub struct Version {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
    /// Textual description of the release, or None if not available or if not UTF-8
    pub describe: Option<&'static str>,
}

impl Version {
    /// Converts the ffi type `bladerf_version` to `Self`.
    ///
    /// # Safety
    /// `version` must come from a bladerf ffi call.
    /// More specifically:
    /// `version.describe` must be a null-terminated, immutable, statically-allocated (always valid),
    /// string.
    pub unsafe fn from_ffi(version: &bladerf_version) -> Self {
        let describe = if !version.describe.is_null() {
            // SAFETY: bladefr docs on field say do not try to modify or free this,
            // which sounds like a static lifetime to me
            let cstr = unsafe { CStr::from_ptr::<'static>(version.describe) };
            cstr.to_str().ok()
        } else {
            None
        };

        Version {
            major: version.major,
            minor: version.minor,
            patch: version.patch,
            describe,
        }
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(desc) = self.describe {
            f.write_fmt(format_args!(
                "v{}.{}.{} ({})",
                self.major, self.minor, self.patch, desc
            ))
        } else {
            f.write_fmt(format_args!(
                "v{}.{}.{}",
                self.major, self.minor, self.patch,
            ))
        }
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let major_ord = self.major.cmp(&other.major);
        if major_ord != cmp::Ordering::Equal {
            return major_ord;
        }
        let minor_ord = self.minor.cmp(&other.minor);
        if minor_ord != cmp::Ordering::Equal {
            return minor_ord;
        }
        self.patch.cmp(&other.patch)
    }
}

impl PartialEq for Version {
    fn eq(&self, other: &Self) -> bool {
        self.major == other.major && self.minor == other.minor && self.patch == other.patch
    }
}

impl Eq for Version {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_cmp() {
        let v1 = Version {
            major: 2,
            minor: 0,
            patch: 0,
            describe: None,
        };
        let v2 = Version {
            major: 1,
            minor: 5,
            patch: 10,
            describe: None,
        };

        assert!(v1 > v2);
        assert!(v2 < v1);

        let v1 = Version {
            major: 1,
            minor: 6,
            patch: 0,
            describe: None,
        };
        let v2 = Version {
            major: 1,
            minor: 5,
            patch: 10,
            describe: None,
        };

        assert!(v1 > v2);
        assert!(v2 < v1);

        let v1 = Version {
            major: 1,
            minor: 5,
            patch: 11,
            describe: None,
        };
        let v2 = Version {
            major: 1,
            minor: 5,
            patch: 10,
            describe: None,
        };

        assert!(v1 > v2);
        assert!(v2 < v1);

        let v1 = Version {
            major: 1,
            minor: 5,
            patch: 10,
            describe: Some("test"),
        };
        let v2 = Version {
            major: 1,
            minor: 5,
            patch: 10,
            describe: Some("another test"),
        };

        assert_eq!(v1, v2);

        let v1 = Version {
            major: 1,
            minor: 5,
            patch: 11,
            describe: None,
        };
        let v2 = Version {
            major: 1,
            minor: 6,
            patch: 0,
            describe: None,
        };
        let v3 = Version {
            major: 2,
            minor: 0,
            patch: 0,
            describe: None,
        };

        assert!(v1 < v2);
        assert!(v2 < v3);
        assert!(v1 < v3);
    }
}
