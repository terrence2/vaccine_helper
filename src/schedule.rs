use anyhow::Result;
use jiff::{SpanRound, Unit, Zoned};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
    fmt,
    sync::OnceLock,
};

// Record the number of months between doses.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum DoseSchedule {
    Single,
    Repeated {
        number: u8,
        interval: i16,
    },
    RepeatedRange {
        number: u8,
        minimum: i16,
        maximum: i16,
    },
}

impl DoseSchedule {
    fn all_doses(&self) -> Vec<(DoseKind, i16)> {
        match self {
            Self::Single => vec![(DoseKind::Dose(0), 0)],
            Self::Repeated { number, interval } => (0u8..*number)
                .map(|i| (DoseKind::Dose(i), i as i16 * interval))
                .collect(),
            Self::RepeatedRange {
                number, minimum, ..
            } => (0u8..*number)
                .map(|i| (DoseKind::Dose(i), i as i16 * minimum))
                .collect(),
        }
    }

    fn minimum_dose_interval(&self) -> i16 {
        match self {
            Self::Single => 0,
            Self::Repeated { interval, .. } => *interval,
            Self::RepeatedRange { minimum, .. } => *minimum,
        }
    }

    // Return the month offsets for all doses we still need to get
    fn all_months<'a>(
        &self,
        now: &Zoned,
        dose_records: impl Iterator<Item = &'a &'a VaccineRecord>,
    ) -> Result<Vec<(DoseKind, i16)>> {
        let dose_records = dose_records.collect::<Vec<_>>();
        if dose_records.is_empty() {
            return Ok(self.all_doses());
        }

        let dose_record_kinds = dose_records
            .iter()
            .map(|record| *record.kind())
            .collect::<HashSet<_>>();

        // Filter all_doses to remove any doses that are already in the records.
        let mut required_doses: Vec<(DoseKind, i16)> = self
            .all_doses()
            .iter()
            .filter(|(kind, _)| !dose_record_kinds.contains(kind))
            .cloned()
            .collect();

        // We might already have all our doses.
        if required_doses.is_empty() {
            return Ok(required_doses);
        }

        // In the complex case, we need to recompute offsets based on what has been received and when.
        assert!(!dose_records.is_empty());
        assert!(!required_doses.is_empty());

        // Get the offset from now to the first does we need. We will need to move all doses forward by this amount.
        let next_dose_mo = required_doses[0].1;

        // Find the offset from our last dose to now. We may need to push doses forward, if the recommended interval
        // has not yet been reached for subsequent doses.
        let last_dose_to_now = &dose_records[0].date - now;
        let last_dose_mo: i16 = last_dose_to_now
            .round(SpanRound::new().smallest(Unit::Month).relative(now))?
            .get_months()
            .try_into()?;
        assert!(last_dose_mo <= 0);
        let min_interval = self.minimum_dose_interval();
        let min_dose_offset = if -last_dose_mo > min_interval {
            0
        } else {
            min_interval + last_dose_mo
        };

        for (_, mo) in required_doses.iter_mut() {
            *mo = *mo - next_dose_mo + min_dose_offset;
        }

        // for (record, (expect_dose_kind, dose_offset)) in dose_records.zip(self.all_doses()) {
        //     if record.kind() == expect_dose_kind {}
        // }
        // if dose_records.count() >= self.num_doses() as usize {
        //     return vec![];
        // }

        // match self {
        //     Self::Single => vec![0],
        //     Self::Repeated { number, interval } => {
        //         (0..*number).map(|i| i as i16 * interval).collect()
        //     }
        //     Self::RepeatedRange {
        //         number, minimum, ..
        //     } => (0..*number).map(|i| i as i16 * minimum).collect(),
        // }

        Ok(required_doses)
    }
}

impl fmt::Display for DoseSchedule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Single => write!(f, "1x"),
            Self::Repeated { number, interval } => write!(f, "{number}x every {interval}mo"),
            Self::RepeatedRange {
                number,
                minimum,
                maximum,
            } => write!(f, "{number}x every {minimum}-{maximum}mo"),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum BoosterSchedule {
    Annual,
    Years(i16),
    Lifetime,
}

impl BoosterSchedule {
    // Return the month offsets for all shots
    fn all_months(&self, last_dose_mo: i16, limit_mo: i16) -> Vec<(DoseKind, i16)> {
        let mut out = Vec::new();
        for mo in 1..limit_mo {
            match self {
                Self::Annual if mo % 12 == 0 => {
                    out.push((DoseKind::Booster, last_dose_mo + mo));
                }
                Self::Years(n) if mo % (12 * n) == 0 => {
                    out.push((DoseKind::Booster, last_dose_mo + mo));
                }
                Self::Lifetime if mo % (12 * 25) == 0 => {
                    out.push((DoseKind::Booster, last_dose_mo + mo));
                }
                _ => {}
            }
        }
        out
    }

    fn duration(&self) -> i16 {
        match self {
            Self::Annual => 12,
            Self::Years(n) => 12 * n,
            Self::Lifetime => 25,
        }
    }
}

impl Ord for BoosterSchedule {
    fn cmp(&self, other: &Self) -> Ordering {
        self.duration().cmp(&other.duration())
    }
}
impl PartialOrd for BoosterSchedule {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl fmt::Display for BoosterSchedule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Annual => write!(f, "every year"),
            Self::Years(n) => write!(f, "every {n} years"),
            Self::Lifetime => write!(f, "every 25-30 years or when exposed"),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Vaccine {
    name: &'static str,
    treats: Vec<&'static str>,
    initial_schedule: DoseSchedule,
    booster_schedule: BoosterSchedule,
    notes: &'static str,
    recommended: bool,
}

impl Ord for Vaccine {
    fn cmp(&self, other: &Self) -> Ordering {
        self.booster_schedule.cmp(&other.booster_schedule)
    }
}

impl PartialOrd for Vaccine {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/*
TODO: need to research all of these and figure out how to default them off.
Typhoid
Rabies
Cholera
Japanese Encephalitis
Chikungunya
Tick-borne Encephalitis
*/

impl Vaccine {
    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn dosage_schedule(&self) -> &DoseSchedule {
        &self.initial_schedule
    }

    pub fn booster_schedule(&self) -> &BoosterSchedule {
        &self.booster_schedule
    }

    pub fn notes(&self) -> &str {
        self.notes
    }

    pub fn recommended(&self) -> bool {
        self.recommended
    }

    pub fn treats_str(&self) -> String {
        self.treats.join(", ")
    }

    // Return all doses that are needed, starting at `now` and taking into account the vaccine
    // history in records. All records are for the current vaccine, but may contain both doses
    // and boosters. When a Dose has been taken in the past, the next dose should be scheduled
    // at the time when it is due, if in the future, or as soon as possible if it is past due,
    // but preserving offsets between subsequent doses after.
    pub fn all_doses<'a>(
        &self,
        now: &Zoned,
        records: impl Iterator<Item = &'a VaccineRecord>,
        end_plan_mo: i16,
    ) -> Result<Vec<(DoseKind, i16)>> {
        let vaccine_records: Vec<&VaccineRecord> = records.collect();
        let dose_records = vaccine_records
            .iter()
            .filter(|record| matches!(record.kind(), DoseKind::Dose(_)));

        // TODO: booster scheduling
        let _booster_records = vaccine_records
            .iter()
            .filter(|record| matches!(record.kind(), DoseKind::Booster));

        let mut initial = self.initial_schedule.all_months(now, dose_records)?;
        let booster = self
            .booster_schedule
            .all_months(initial.last().map(|v| v.1).unwrap_or(0), end_plan_mo);
        initial.extend(booster);
        Ok(initial)
    }

    pub fn get_vaccines() -> &'static HashMap<&'static str, Vaccine> {
        static VACCINES: OnceLock<HashMap<&'static str, Vaccine>> = OnceLock::new();
        VACCINES.get_or_init(|| HashMap::from_iter([
            ("COVID-19", Vaccine {
                name: "COVID-19",
                treats: vec!["COVID-19"],
                initial_schedule: DoseSchedule::RepeatedRange { number: 2, minimum: 1, maximum: 2 },
                booster_schedule: BoosterSchedule::Annual,
                notes: "Get a booster in Sept/Oct to catch any new variants.",
                recommended: true,
            }),
            ("Flu", Vaccine {
                name: "Flu",
                treats: vec!["Flu"],
                initial_schedule: DoseSchedule::Single,
                booster_schedule: BoosterSchedule::Annual,
                notes: "Get a booster in Sept/Oct to catch any new variants. Get a second dose in the middle of the season if you have no prior exposure.",
                recommended: true,
            }),
            ("Tdap", Vaccine {
                name: "Tdap",
                treats: vec!["Tuberculosis", "Tetanus", "Diphtheria", "Pertussis"],
                initial_schedule: DoseSchedule::Repeated { number: 3, interval: 6 },
                booster_schedule: BoosterSchedule::Years(10),
                notes: "Tuberculosis is humanity's greatest adversary; please do your part by getting vaccinated and staying up to date with boosters!",
                recommended: true,
            }),
            ("Mpox", Vaccine {
                name: "Mpox",
                treats: vec!["Monkeypox", "Smallpox"],
                initial_schedule: DoseSchedule::RepeatedRange { number: 2, minimum: 1, maximum: 6 },
                booster_schedule: BoosterSchedule::Years(5),
                notes: "The 'M' is for both \"Monkey\" and Small",
                recommended: true,
            }),
            ("Meningitis", Vaccine {
                name: "Meningitis",
                treats: vec!["Meningitis"],
                initial_schedule: DoseSchedule::Repeated { number: 2, interval: 6 },
                booster_schedule: BoosterSchedule::Years(5),
                notes: "Only recommended for adults that are exposed regularly, but low risk to get it so why not?",
                recommended: true,
            }),
            ("MMR", Vaccine {
                name: "MMR",
                treats: vec!["Measles", "Mumps", "Rubella"],
                initial_schedule: DoseSchedule::Repeated { number: 2, interval: 5 * 12 },
                booster_schedule: BoosterSchedule::Years(5),
                notes: "Recommended for children and immuno-compromised, but again low risk so why not? Note: measles and rubella are lifetime immunity, but mumps requires a 5 year booster.",
                recommended: true,
            }),
            ("Shinglex", Vaccine {
                name: "Shinglex",
                treats: vec!["Shingles"],
                initial_schedule: DoseSchedule::RepeatedRange { number: 2, minimum: 2, maximum: 6 },
                booster_schedule: BoosterSchedule::Years(7),
                notes: "Recommended for children and immuno-compromised, but again low risk so why not?",
                recommended: true,
            }),
            ("PCV20", Vaccine {
                name: "PCV20",
                treats: vec!["Pneumonia"],
                initial_schedule: DoseSchedule::Repeated { number: 2, interval: 6 },
                booster_schedule: BoosterSchedule::Lifetime,
                notes: "Recommended for at risk and 50+, but no risk to get it sooner, so why not?",
                recommended: true,
            }),
            ("Gardacil-9", Vaccine {
                name: "Gardacil-9",
                treats: vec!["Human Papillomavirus (HPV)"],
                initial_schedule: DoseSchedule::Repeated { number: 3, interval: 6 },
                booster_schedule: BoosterSchedule::Lifetime,
                notes: "HPV causes cancer in men and women both. Don't ignore it just because you haven't been specifically advertised to.",
                recommended: true,
            }),
            ("Hepatitis B", Vaccine {
                name: "Hepatitis B",
                treats: vec!["Hepatitis B"],
                initial_schedule: DoseSchedule::Single,
                booster_schedule: BoosterSchedule::Lifetime,
                notes: "Greater than 30 years proven durability. Definitely worth it.",
                recommended: true,
            }),
            ("Hepatitis A", Vaccine {
                name: "Hepatitis A",
                treats: vec!["Hepatitis A"],
                initial_schedule: DoseSchedule::Repeated { number: 2, interval: 6 },
                booster_schedule: BoosterSchedule::Lifetime,
                notes: "Greater than 25 years proven durability. Definitely worth it.",
                recommended: true,
            }),
            ("Hepatitis A&B", Vaccine {
                name: "Hepatitis A&B",
                treats: vec!["Hepatitis A", "Hepatitis B"],
                initial_schedule: DoseSchedule::Repeated { number: 3, interval: 6 },
                booster_schedule: BoosterSchedule::Lifetime,
                notes: "Not recommended for adults despite hepA/hepB being individually recommended. 🤷",
                recommended: false,
            }),
            ("IPV", Vaccine {
                name: "IPV",
                treats: vec!["Polio"],
                initial_schedule: DoseSchedule::Repeated { number: 4, interval: 4 },
                booster_schedule: BoosterSchedule::Lifetime,
                notes: "No recommendation for adults, but get a booster if you're at risk or risk averse.",
                recommended: true,
            }),
            ("Chickenpox", Vaccine {
                name: "Chickenpox",
                treats: vec!["Chickenpox"],
                initial_schedule: DoseSchedule::RepeatedRange { number: 2, minimum: 1, maximum: 6 },
                booster_schedule: BoosterSchedule::Lifetime,
                notes: "Recommended if at risk or haven't had chickenpox yet, but low risk so why not?",
                recommended: true,
            })]))
    }

    // Schedule all vaccines listed in `prio` until `end_plan_year`.
    // Limit weekly shots to nshots.
    //
    // Note: track everything internally in months offset from now and only convert to
    //       real times with now base when we commit to an appointment.
    //
    // TODO: allow for some shots to have happened already. Need a record struct.
    pub fn schedule(
        now: &Zoned,
        prio: impl Iterator<Item = String>,
        end_plan_year: i16,
        records: &[VaccineRecord],
    ) -> Result<Vec<VaccineAppointment>> {
        // Compute mo offset from current to end schedule at.
        let current_year = now.year();
        let limit_mo = (end_plan_year - current_year) * 12;

        let vaccines = Vaccine::get_vaccines();
        let mut appointments = Vec::new();
        for vaccine_name in prio {
            let vaccine = vaccines.get(vaccine_name.as_str()).unwrap();
            let vaccine_records = records.iter().filter(|r| r.vaccine() == vaccine.name);
            for (kind, dose_mo) in vaccine.all_doses(now, vaccine_records, limit_mo)? {
                appointments.push(VaccineAppointment::from_month_offset(
                    vaccine.name(),
                    kind,
                    now,
                    dose_mo,
                ))
            }
        }
        appointments.sort();
        Ok(appointments)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub enum DoseKind {
    Dose(u8),
    #[default]
    Booster,
}

impl Display for DoseKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Dose(index) => write!(f, "Dose#{}", index + 1),
            Self::Booster => write!(f, "Booster"),
        }
    }
}

impl Ord for DoseKind {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Self::Dose(a), Self::Dose(b)) => a.cmp(b),
            (Self::Dose(_), Self::Booster) => Ordering::Less,
            (Self::Booster, Self::Dose(_)) => Ordering::Greater,
            (Self::Booster, Self::Booster) => Ordering::Equal,
        }
    }
}
impl PartialOrd for DoseKind {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl DoseKind {
    pub fn all_kinds() -> &'static [(&'static str, DoseKind)] {
        static NAMES: OnceLock<&'static [(&'static str, DoseKind)]> = OnceLock::new();
        NAMES.get_or_init(|| {
            &[
                ("Booster", Self::Booster),
                ("Dose#1", Self::Dose(0)),
                ("Dose#2", Self::Dose(1)),
                ("Dose#3", Self::Dose(2)),
                ("Dose#4", Self::Dose(3)),
            ]
        })
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct VaccineRecord {
    vaccine: String,
    date: Zoned,
    kind: DoseKind,
    notes: String,
}

impl Default for VaccineRecord {
    fn default() -> Self {
        Self {
            vaccine: "Tdap".into(),
            date: Zoned::now(),
            kind: DoseKind::Booster,
            notes: String::new(),
        }
    }
}

impl VaccineRecord {
    pub fn vaccine(&self) -> &str {
        &self.vaccine
    }

    pub fn vaccine_mut(&mut self) -> &mut String {
        &mut self.vaccine
    }

    pub fn date(&self) -> &Zoned {
        &self.date
    }

    pub fn date_mut(&mut self) -> &mut Zoned {
        &mut self.date
    }

    pub fn kind(&self) -> &DoseKind {
        &self.kind
    }

    pub fn kind_mut(&mut self) -> &mut DoseKind {
        &mut self.kind
    }

    pub fn notes(&self) -> &str {
        &self.notes
    }

    pub fn notes_mut(&mut self) -> &mut String {
        &mut self.notes
    }

    // pub fn have_matching(records: &[VaccineRecord], vaccine_name: &str, index: u8) -> bool {
    //     records
    //         .iter()
    //         .any(|record| record.vaccine == vaccine_name && record.kind == DoseKind::Dose(index))
    // }
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub struct VaccineAppointment {
    vaccine: String,
    kind: DoseKind,
    year: i16,
    month: i8,
}

impl VaccineAppointment {
    pub fn vaccine(&self) -> &str {
        &self.vaccine
    }

    pub fn kind(&self) -> DoseKind {
        self.kind
    }

    pub fn year(&self) -> i16 {
        self.year
    }

    pub fn month(&self) -> i8 {
        self.month
    }

    fn from_month_offset(vaccine: &str, kind: DoseKind, now: &Zoned, mo: i16) -> Self {
        let (year, month) = Self::mo_to_ym(now, mo);
        VaccineAppointment {
            vaccine: vaccine.to_string(),
            kind,
            year,
            month,
        }
    }

    fn mo_to_ym(now: &Zoned, mo: i16) -> (i16, i8) {
        // guaranteed to be in range 1..=12
        let month = now.date().month();
        let year = now.date().year();
        // note: move to 0-based month offsets so we can div and mod easily.
        let month_offset = month as i16 + mo - 1;
        let year_offset = month_offset / 12;
        let month: i8 = ((month_offset % 12) + 1).try_into().unwrap();
        assert!((1..=12).contains(&month));
        (year.saturating_add(year_offset), month)
    }
}

impl Ord for VaccineAppointment {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.year != other.year {
            return self.year.cmp(&other.year);
        }
        self.month.cmp(&other.month)
    }
}
impl PartialOrd for VaccineAppointment {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use jiff::{
        civil::{date, Date},
        tz::TimeZone,
        Span,
    };
    use std::ops::Sub;

    fn test_time() -> Result<Zoned> {
        Ok(Date::new(2025, 6, 1)?.to_zoned(TimeZone::get("America/Los_Angeles")?)?)
    }

    #[test]
    fn test_mo_to_ym() -> Result<()> {
        assert_eq!((2025, 6), VaccineAppointment::mo_to_ym(&test_time()?, 0));
        assert_eq!((2025, 7), VaccineAppointment::mo_to_ym(&test_time()?, 1));
        assert_eq!((2025, 8), VaccineAppointment::mo_to_ym(&test_time()?, 2));
        assert_eq!((2025, 9), VaccineAppointment::mo_to_ym(&test_time()?, 3));
        assert_eq!((2025, 10), VaccineAppointment::mo_to_ym(&test_time()?, 4));
        assert_eq!((2025, 11), VaccineAppointment::mo_to_ym(&test_time()?, 5));
        assert_eq!((2025, 12), VaccineAppointment::mo_to_ym(&test_time()?, 6));
        assert_eq!((2026, 1), VaccineAppointment::mo_to_ym(&test_time()?, 7));
        Ok(())
    }

    #[test]
    fn test_dosing_schedule_basic() -> Result<()> {
        assert_eq!(
            vec![
                (DoseKind::Dose(0), 0i16),
                (DoseKind::Dose(1), 6),
                (DoseKind::Dose(2), 12)
            ],
            Vaccine::get_vaccines()
                .get("Tdap")
                .unwrap()
                .dosage_schedule()
                .all_months(&test_time()?, [].iter())?
        );
        assert_eq!(
            vec![(DoseKind::Dose(0), 0i16), (DoseKind::Dose(1), 1)],
            Vaccine::get_vaccines()
                .get("Mpox")
                .unwrap()
                .dosage_schedule()
                .all_months(&test_time()?, [].iter())?
        );
        Ok(())
    }

    #[test]
    fn test_dosing_schedule_with_records() -> Result<()> {
        // Last dose long enough ago that we don't need to offset to meet minimum intervals
        assert_eq!(
            vec![(DoseKind::Dose(1), 0), (DoseKind::Dose(2), 6)],
            Vaccine::get_vaccines()
                .get("Tdap")
                .unwrap()
                .dosage_schedule()
                .all_months(
                    &test_time()?,
                    [&&VaccineRecord {
                        vaccine: "Tdap".to_string(),
                        date: test_time()?.sub(Span::new().months(7)),
                        kind: DoseKind::Dose(0),
                        notes: "".to_string(),
                    }]
                    .into_iter()
                )?
        );
        // Last dose close enough that we need to offset some to meet minimum intervals
        assert_eq!(
            vec![(DoseKind::Dose(1), 1), (DoseKind::Dose(2), 7)],
            Vaccine::get_vaccines()
                .get("Tdap")
                .unwrap()
                .dosage_schedule()
                .all_months(
                    &test_time()?,
                    [&&VaccineRecord {
                        vaccine: "Tdap".to_string(),
                        date: test_time()?.sub(Span::new().months(5)),
                        kind: DoseKind::Dose(0),
                        notes: "".to_string(),
                    }]
                    .into_iter()
                )?
        );
        Ok(())
    }

    #[test]
    fn test_booster_months() -> Result<()> {
        // assert_eq!(
        //     vec![60],
        //     Vaccine::get_vaccines()
        //         .get("Mpox")
        //         .unwrap()
        //         .booster_schedule()
        //         .all_months(0, 5 * 12 + 1)
        // );
        Ok(())
    }
}

// pub struct ReceivedDose {
//     vaccine: String,
//     date: Date,
//     location: String,
//     notes: String,
// }
