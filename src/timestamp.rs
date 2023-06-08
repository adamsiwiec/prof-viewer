use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Default, Deserialize, Serialize)]
pub struct Timestamp(pub i64 /* ns */);

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Time is stored in nanoseconds. But display in larger units if possible.
        let ns = self.0;
        let ns_per_us = 1_000;
        let ns_per_ms = 1_000_000;
        let ns_per_s = 1_000_000_000;
        let divisor;
        let remainder_divisor;
        let mut unit_name = "ns";
        if ns >= ns_per_s {
            divisor = ns_per_s;
            remainder_divisor = divisor / 1_000;
            unit_name = "s";
        } else if ns >= ns_per_ms {
            divisor = ns_per_ms;
            remainder_divisor = divisor / 1_000;
            unit_name = "ms";
        } else if ns >= ns_per_us {
            divisor = ns_per_us;
            remainder_divisor = divisor / 1_000;
            unit_name = "us";
        } else {
            return write!(f, "{ns} {unit_name}");
        }
        let units = ns / divisor;
        let remainder = (ns % divisor) / remainder_divisor;
        write!(f, "{units}.{remainder:0>3} {unit_name}")
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Default, Deserialize, Serialize)]
pub struct Interval {
    pub start: Timestamp,
    pub stop: Timestamp, // exclusive
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, Copy)]
pub enum IntervalParseError {
    NoValue,
    InvalidValue,
    NoUnit,
    InvalidUnit,
    StartAfterStop,
    StartAfterEnd,
    StopBeforeStart,
}

impl fmt::Display for IntervalParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IntervalParseError::NoValue => write!(f, "no value"),
            IntervalParseError::InvalidValue => write!(f, "invalid value"),
            IntervalParseError::NoUnit => write!(f, "no unit"),
            IntervalParseError::InvalidUnit => write!(f, "invalid unit"),
            IntervalParseError::StartAfterStop => write!(f, "start after stop"),
            IntervalParseError::StartAfterEnd => write!(f, "start after end"),
            IntervalParseError::StopBeforeStart => write!(f, "stop before start"),
        }
    }
}

impl fmt::Display for Interval {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Time is stored in nanoseconds. But display in larger units if possible.
        let start_ns = self.start.0;
        let stop_ns = self.stop.0;
        let ns_per_us = 1_000;
        let ns_per_ms = 1_000_000;
        let ns_per_s = 1_000_000_000;
        let divisor;
        let remainder_divisor;
        let mut unit_name = "ns";
        if stop_ns >= ns_per_s {
            divisor = ns_per_s;
            remainder_divisor = divisor / 1_000;
            unit_name = "s";
        } else if stop_ns >= ns_per_ms {
            divisor = ns_per_ms;
            remainder_divisor = divisor / 1_000;
            unit_name = "ms";
        } else if stop_ns >= ns_per_us {
            divisor = ns_per_us;
            remainder_divisor = divisor / 1_000;
            unit_name = "us";
        } else {
            return write!(
                f,
                "from {} to {} {} (duration: {})",
                start_ns,
                stop_ns,
                unit_name,
                Timestamp(self.duration_ns())
            );
        }
        let start_units = start_ns / divisor;
        let start_remainder = (start_ns % divisor) / remainder_divisor;
        let stop_units = stop_ns / divisor;
        let stop_remainder = (stop_ns % divisor) / remainder_divisor;
        write!(
            f,
            "from {}.{:0>3} to {}.{:0>3} {} (duration: {})",
            start_units,
            start_remainder,
            stop_units,
            stop_remainder,
            unit_name,
            Timestamp(self.duration_ns())
        )
    }
}

impl Interval {
    pub fn new(start: Timestamp, stop: Timestamp) -> Self {
        Self { start, stop }
    }
    pub fn duration_ns(self) -> i64 {
        self.stop.0 - self.start.0
    }
    pub fn contains(self, point: Timestamp) -> bool {
        point >= self.start && point < self.stop
    }
    pub fn overlaps(self, other: Interval) -> bool {
        !(other.stop < self.start || other.start >= self.stop)
    }
    pub fn intersection(self, other: Interval) -> Self {
        Self {
            start: Timestamp(self.start.0.max(other.start.0)),
            stop: Timestamp(self.stop.0.min(other.stop.0)),
        }
    }
    pub fn union(self, other: Interval) -> Self {
        Self {
            start: Timestamp(self.start.0.min(other.start.0)),
            stop: Timestamp(self.stop.0.max(other.stop.0)),
        }
    }
    // Convert a timestamp into [0,1] relative space
    pub fn unlerp(self, time: Timestamp) -> f32 {
        (time.0 - self.start.0) as f32 / (self.duration_ns() as f32)
    }
    // Convert [0,1] relative space into a timestamp
    pub fn lerp(self, value: f32) -> Timestamp {
        Timestamp((value * (self.duration_ns() as f32)).round() as i64 + self.start.0)
    }

    // convert a string like "500.0 s" to a timestamp
    pub fn parse_timestamp(s: &str) -> Result<Timestamp, IntervalParseError> {
        let mut parts = s.split_whitespace();
        let prefix: &str = parts.next().ok_or(IntervalParseError::NoValue)?;
        let mut unit = prefix.trim_start_matches(|c: char| c.is_numeric() || c == '.');
        let value: &str = prefix.trim_end_matches(|c: char| c.is_alphabetic());
        if value.is_empty() {
            return Err(IntervalParseError::NoValue);
        }
        let value = value
            .parse::<f64>()
            .map_err(|_| IntervalParseError::InvalidValue)?;
        if unit.is_empty() {
            unit = parts.next().ok_or(IntervalParseError::NoUnit)?;
        }

        if parts.next().is_some() {
            return Err(IntervalParseError::InvalidValue);
        }
        let unit = unit.to_lowercase();
        let ns_per_us = 1_000;
        let ns_per_ms = 1_000_000;
        let ns_per_s = 1_000_000_000;
        let ns = match unit.as_str() {
            "ns" => value as i64,
            "us" => (value * ns_per_us as f64) as i64,
            "ms" => (value * ns_per_ms as f64) as i64,
            "s" => (value * ns_per_s as f64) as i64,
            _ => return Err(IntervalParseError::InvalidUnit),
        };
        Ok(Timestamp(ns))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // test all the different ways to parse a timestamp
    #[test]
    fn test_ms() {
        assert_eq!(
            Interval::parse_timestamp("500.0 ms").unwrap(),
            Timestamp(500_000_000)
        );
    }

    #[test]
    fn test_us() {
        assert_eq!(
            Interval::parse_timestamp("500.0 us").unwrap(),
            Timestamp(500_000)
        );
    }

    #[test]
    fn test_ns() {
        assert_eq!(
            Interval::parse_timestamp("500.0 ns").unwrap(),
            Timestamp(500)
        );
    }

    #[test]
    fn test_s() {
        assert_eq!(
            Interval::parse_timestamp("500.0 s").unwrap(),
            Timestamp(500_000_000_000)
        );
    }

    #[test]
    fn test_no_unit() {
        assert_eq!(
            Interval::parse_timestamp("500.0").unwrap_err(),
            IntervalParseError::NoUnit
        );
    }

    #[test]
    fn test_no_value() {
        assert_eq!(
            Interval::parse_timestamp("ms").unwrap_err(),
            IntervalParseError::NoValue
        );
    }

    #[test]
    fn test_invalid_unit() {
        assert_eq!(
            Interval::parse_timestamp("500.0 foo").unwrap_err(),
            IntervalParseError::InvalidUnit
        );
    }

    #[test]
    fn test_invalid_value() {
        assert_eq!(
            Interval::parse_timestamp("foo ms").unwrap_err(),
            IntervalParseError::NoValue
        );
    }

    #[test]
    fn test_invalid_value2() {
        assert_eq!(
            Interval::parse_timestamp("500.0.0 ms").unwrap_err(),
            IntervalParseError::InvalidValue
        );
    }

    #[test]
    fn test_invalid_value3() {
        assert_eq!(
            Interval::parse_timestamp("500.0.0").unwrap_err(),
            IntervalParseError::InvalidValue
        );
    }

    #[test]
    fn test_extra() {
        assert_eq!(
            Interval::parse_timestamp("500.0 ms asdfadf").unwrap_err(),
            IntervalParseError::InvalidValue
        );
    }
}
