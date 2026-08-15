//! Dock families — tape · eye · water · weird.

use super::Look;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LookFamily {
    Tape,
    Eye,
    Water,
    Weird,
}

impl LookFamily {
    pub const ALL: [Self; 4] = [Self::Tape, Self::Eye, Self::Water, Self::Weird];

    pub fn label(self) -> &'static str {
        match self {
            Self::Tape => "TAPE",
            Self::Eye => "EYE",
            Self::Water => "WATER",
            Self::Weird => "WEIRD",
        }
    }

    pub fn chip(self) -> &'static str {
        match self {
            Self::Tape => "tape",
            Self::Eye => "eye",
            Self::Water => "water",
            Self::Weird => "weird",
        }
    }

    pub fn rail(self) -> &'static [Look] {
        match self {
            Self::Tape => &TAPE_RAIL,
            Self::Eye => &EYE_RAIL,
            Self::Water => &WATER_RAIL,
            Self::Weird => &WEIRD_RAIL,
        }
    }
}

impl Look {
    pub fn family(self) -> LookFamily {
        match self {
            Look::None => LookFamily::Tape,
            Look::Vhs | Look::Gx | Look::Uhf | Look::Beta | Look::D8 | Look::Live | Look::Sat
            | Look::Mosh | Look::Glitch | Look::Datamosh => LookFamily::Tape,
            Look::Cctv | Look::Thermal | Look::Xray | Look::Noir | Look::Cyber
            | Look::Slitscan => LookFamily::Eye,
            Look::Ripple | Look::Smear | Look::Breathe | Look::Film | Look::Waves | Look::Fluid
            | Look::Drift | Look::Reaction => LookFamily::Water,
            _ => LookFamily::Weird,
        }
    }

    pub fn index_in_family(self) -> usize {
        self.family()
            .rail()
            .iter()
            .position(|&l| l == self)
            .unwrap_or(0)
    }
}

const TAPE_RAIL: [Look; 11] = [
    Look::None,
    Look::Vhs,
    Look::Gx,
    Look::Uhf,
    Look::Beta,
    Look::D8,
    Look::Live,
    Look::Sat,
    Look::Mosh,
    Look::Glitch,
    Look::Datamosh,
];

const EYE_RAIL: [Look; 7] = [
    Look::None,
    Look::Cctv,
    Look::Thermal,
    Look::Xray,
    Look::Noir,
    Look::Cyber,
    Look::Slitscan,
];

const WATER_RAIL: [Look; 9] = [
    Look::None,
    Look::Ripple,
    Look::Smear,
    Look::Breathe,
    Look::Film,
    Look::Waves,
    Look::Fluid,
    Look::Drift,
    Look::Reaction,
];

const WEIRD_RAIL: [Look; 20] = [
    Look::None,
    Look::Morph,
    Look::Haunt,
    Look::Smudge,
    Look::Possess,
    Look::Lurk,
    Look::Corrupt,
    Look::Specter,
    Look::Crawl,
    Look::Holo,
    Look::Particles,
    Look::Stamp,
    Look::Echo,
    Look::Chrome,
    Look::Bounce,
    Look::Prism,
    Look::Strata,
    Look::Voronoi,
    Look::Topo,
    Look::Quantum,
];

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn every_look_in_one_family_rail() {
        let mut seen = HashSet::new();
        for family in LookFamily::ALL {
            for &look in family.rail() {
                if look.is_none() {
                    continue;
                }
                assert!(
                    seen.insert(look),
                    "{look:?} appears in more than one family rail"
                );
            }
        }
        for look in Look::RAIL {
            if look.is_none() {
                continue;
            }
            assert!(seen.contains(&look), "{look:?} missing from family rails");
        }
    }
}
