use std::cmp::Ordering;
use std::fmt::{self, Display};
use std::sync::LazyLock;

use regex::Regex;

static UNITY_VERSION_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"([0-9]+)\.([0-9]+)\.([0-9]+)\.?([abcfpx]+)([0-9]+)").unwrap());

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct UnityVersion {
    /// Major release number
    pub major: u16,
    /// Minor release number
    pub minor: u8,
    /// Build release number
    pub build: u8,
    /// Release type
    pub r#type: char,
    /// Release type number
    pub type_number: u8,
}

impl Display for UnityVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Unity {}", self.version())
    }
}

impl PartialOrd for UnityVersion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for UnityVersion {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.major > other.major {
            return Ordering::Greater;
        } else if self.major < other.major {
            return Ordering::Less;
        }
        if self.minor > other.minor {
            return Ordering::Greater;
        } else if self.minor < other.minor {
            return Ordering::Less;
        }
        if self.build > other.build {
            return Ordering::Greater;
        } else if self.build < other.build {
            return Ordering::Less;
        }
        match self.r#type {
            'a' => match other.r#type {
                'a' => {
                    if self.type_number > other.type_number {
                        return Ordering::Greater;
                    } else if self.type_number < other.type_number {
                        return Ordering::Less;
                    }
                    return Ordering::Equal;
                }
                _ => return Ordering::Less,
            },
            'b' => match other.r#type {
                'f' | 'p' => return Ordering::Less,
                'a' => return Ordering::Greater,
                'b' => {
                    if self.type_number > other.type_number {
                        return Ordering::Greater;
                    } else if self.type_number < other.type_number {
                        return Ordering::Less;
                    }
                    return Ordering::Equal;
                }
                _ => unreachable!(),
            },
            'p' => match other.r#type {
                'a' | 'b' => return Ordering::Greater,
                'p' => {
                    if self.type_number > other.type_number {
                        return Ordering::Greater;
                    } else if self.type_number < other.type_number {
                        return Ordering::Less;
                    }
                    return Ordering::Equal;
                }
                'f' => return Ordering::Less,
                _ => unreachable!(),
            },
            'f' => match other.r#type {
                'a' | 'b' | 'p' => return Ordering::Greater,
                'f' => {
                    if self.type_number > other.type_number {
                        return Ordering::Greater;
                    } else if self.type_number < other.type_number {
                        return Ordering::Less;
                    }
                    return Ordering::Equal;
                }
                _ => unreachable!(),
            },
            _ => return Ordering::Equal,
        }
    }
}

impl From<&str> for UnityVersion {
    fn from(value: &str) -> Self {
        let caps = UNITY_VERSION_REGEX.captures(value).unwrap();

        UnityVersion {
            major: caps.get(1).unwrap().as_str().parse().unwrap(),
            minor: caps.get(2).unwrap().as_str().parse().unwrap(),
            build: caps.get(3).unwrap().as_str().parse().unwrap(),
            r#type: caps.get(4).unwrap().as_str().chars().nth(0).unwrap(),
            type_number: caps.get(5).unwrap().as_str().parse().unwrap(),
        }
    }
}

impl UnityVersion {
    pub fn new(major: u16, minor: Option<u8>, build: Option<u8>, r#type: Option<char>, type_number: Option<u8>) -> Self {
        Self {
            major: major,
            minor: minor.unwrap_or(0),
            build: build.unwrap_or(0),
            r#type: r#type.unwrap_or('a'),
            type_number: type_number.unwrap_or(0),
        }
    }

    #[inline]
    pub fn version(&self) -> String {
        format!("{}.{}.{}{}{}", self.major, self.minor, self.build, self.r#type, self.type_number)
    }
}
