#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Interval {
    M1,
    M5,
    M15,
    H1,
}

impl Interval {
    pub fn minutes(&self) -> u32 {
        match self {
            Interval::M1 => 1,
            Interval::M5 => 5,
            Interval::M15 => 15,
            Interval::H1 => 60,
        }
    }

    pub fn seconds(&self) -> u32 {
        self.minutes() * 60
    }

    pub fn slug(&self) -> &'static str {
        match self {
            Interval::M1 => "1M",
            Interval::M5 => "5M",
            Interval::M15 => "15M",
            Interval::H1 => "1H",
        }
    }

    pub fn up(&self) -> Interval {
        match self {
            Interval::M1 => Interval::M5,
            Interval::M5 => Interval::M15,
            Interval::M15 => Interval::H1,
            Interval::H1 => Interval::H1,
        }
    }

    pub fn down(&self) -> Interval {
        match self {
            Interval::H1 => Interval::M15,
            Interval::M15 => Interval::M5,
            Interval::M5 => Interval::M1,
            Interval::M1 => Interval::M1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Interval;

    #[test]
    fn compare_intervals() {
        let a = Interval::M5;
        let b = Interval::M15;
        assert!(a < b);
        assert_eq!(Interval::M1.up(), Interval::M5);
    }
}
