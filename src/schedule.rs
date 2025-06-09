use anyhow::Result;
use jiff::{SpanRound, Unit, Zoned};
use serde::{Deserialize, Serialize};
use std::{cmp::Ordering, collections::HashMap, fmt, sync::OnceLock};

// Record the number of months between doses.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum DoseSchedule {
    Single,
    Repeated {
        number: u32,
        interval: u32,
    },
    RepeatedRange {
        number: u32,
        minimum: u32,
        maximum: u32,
    },
    Todo,
}

impl DoseSchedule {
    // Return the month offsets for all shots
    fn all_months(&self) -> Vec<u32> {
        match self {
            Self::Single => vec![0],
            Self::Repeated { number, interval } => (0..*number).map(|i| i * interval).collect(),
            Self::RepeatedRange {
                number, minimum, ..
            } => (0..*number).map(|i| i * minimum).collect(),
            Self::Todo => vec![],
        }
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
            Self::Todo => write!(f, "check the CDC website for instructions"),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum BoosterSchedule {
    Annual,
    Years(u32),
    Lifetime,
    Todo,
}

impl BoosterSchedule {
    // Return the month offsets for all shots
    fn all_months(&self, last_dose_mo: u32, limit_mo: u32) -> Vec<u32> {
        let mut out = Vec::new();
        for mo in 1..limit_mo {
            match self {
                Self::Annual if mo % 12 == 0 => {
                    out.push(last_dose_mo + mo);
                }
                Self::Years(n) if mo % (12 * n) == 0 => {
                    out.push(last_dose_mo + mo);
                }
                Self::Lifetime if mo % (12 * 25) == 0 => {
                    out.push(last_dose_mo + mo);
                }
                Self::Todo => return vec![],
                _ => {}
            }
        }
        out
    }

    fn duration(&self) -> u32 {
        match self {
            Self::Annual => 12,
            Self::Years(n) => 12 * n,
            Self::Lifetime => 25,
            Self::Todo => 20,
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
            Self::Todo => write!(f, "check the CDC website for instructions"),
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
COVID-19,
Flu,

5-10 years efficacy
Tdap,

7-10 Years
TODO:
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

    pub fn treats_str(&self) -> String {
        self.treats.join(", ")
    }

    pub fn get_vaccines() -> &'static HashMap<&'static str, Vaccine> {
        static VACCINES: OnceLock<HashMap<&'static str, Vaccine>> = OnceLock::new();
        VACCINES.get_or_init(|| HashMap::from_iter([
            // Annual
            ("COVID-19", Vaccine {
                name: "COVID-19",
                treats: vec!["COVID-19"],
                initial_schedule: DoseSchedule::Todo,
                booster_schedule: BoosterSchedule::Annual,
                notes: "Get a booster in Sept/Oct to catch any new variants."
            }),
            ("Flu", Vaccine {
                name: "Flu",
                treats: vec!["Flu"],
                initial_schedule: DoseSchedule::Todo,
                booster_schedule: BoosterSchedule::Annual,
                notes: "Get a booster in Sept/Oct to catch any new variants."
            }),
            // 5-10 years
            ("Tdap", Vaccine {
                name: "Tdap",
                treats: vec!["Tuberculosis", "Tetanus", "Diphtheria", "Pertussis"],
                initial_schedule: DoseSchedule::Repeated { number: 3, interval: 6 },
                booster_schedule: BoosterSchedule::Todo,
                notes: "",
            }),
            // Mpox (M is for Monkey and Small) [2x doses 1mo apart; 5 year boosters after],
            ("Mpox", Vaccine {
                name: "Mpox",
                treats: vec!["Monkeypox", "Smallpox"],
                initial_schedule: DoseSchedule::RepeatedRange { number: 2, minimum: 1, maximum: 6 },
                booster_schedule: BoosterSchedule::Years(5),
                notes: "The 'M' is for both \"Monkey\" and Small",
            }),
            ("Meningitis", Vaccine {
                name: "Meningitis",
                treats: vec!["Meningitis"],
                initial_schedule: DoseSchedule::Repeated { number: 2, interval: 6 },
                booster_schedule: BoosterSchedule::Years(5),
                notes: "Only recommended for adults that are exposed regularly, but low risk to get it so why not?",
            }),
            // MMR [2x doses 5 years apart, may need to re-dose for mumps every 5 years, if that's ever a thing again],
            ("MMR", Vaccine {
                name: "MMR",
                treats: vec!["Measles", "Mumps", "Rubella"],
                initial_schedule: DoseSchedule::Repeated { number: 2, interval: 5 * 12 },
                booster_schedule: BoosterSchedule::Years(5),
                notes: "Recommended for children and immuno-compromised, but again low risk so why not? Note: measles and rubella are lifetime immunity, but mumps requires a 5 year booster.",
            }),
            // 7-10 years
// Shingles (Shinglex) [2x doses 2-6mo apart with 10 year boosters or closer if at risk; recommended for children and immuno-compromised, but again low risk so why not],
            ("Shinglex", Vaccine {
                name: "Shinglex",
                treats: vec!["Shingles"],
                initial_schedule: DoseSchedule::RepeatedRange { number: 2, minimum: 2, maximum: 6 },
                booster_schedule: BoosterSchedule::Years(7),
                notes: "Recommended for children and immuno-compromised, but again low risk so why not?",
            }),
            // Lifetime, probably? So far, herd immunity has made it impossible to research long-term efficacy. So there's another silver lining for 2025, I guess?
// Pneumonia [2x doses 6mo apart; recommended for at risk and 50+, but no risk to get it sooner, so why not] (PCV20),
            ("PCV20", Vaccine {
                name: "PCV20",
                treats: vec!["Pneumonia"],
                initial_schedule: DoseSchedule::Repeated { number: 2, interval: 6 },
                booster_schedule: BoosterSchedule::Lifetime,
                notes: "Recommended for at risk and 50+, but no risk to get it sooner, so why not?",
            }),
            // HPV [3x doses every 6mo for 1.5 years; lower efficacy if over 25, but why not] (Gardacil-9),
            ("Gardacil-9", Vaccine {
                name: "Gardacil-9",
                treats: vec!["Human Papillomavirus (HPV)"],
                initial_schedule: DoseSchedule::Repeated { number: 3, interval: 6 },
                booster_schedule: BoosterSchedule::Lifetime,
                notes: "",
            }),
// Hepatitis B [single dose; >30 years proven durability],
            ("Hepatitis B", Vaccine {
                name: "Hepatitis B",
                treats: vec!["Hepatitis B"],
                initial_schedule: DoseSchedule::Single,
                booster_schedule: BoosterSchedule::Lifetime,
                notes: "Greater than 30 years proven durability. Definitely worth it.",
            }),
// Hepatitis A [2 doses 6mo apart; >25 years proven durability],
            ("Hepatitis A", Vaccine {
                name: "Hepatitis A",
                treats: vec!["Hepatitis A"],
                initial_schedule: DoseSchedule::Repeated { number: 2, interval: 6 },
                booster_schedule: BoosterSchedule::Lifetime,
                notes: "Greater than 25 years proven durability. Definitely worth it.",
            }),
// Hepatitis A&B [3 doses; not recommended for adults despite hepA/hepB individually recommended 🤷],
            ("Hepatitis A&B", Vaccine {
                name: "Hepatitis A&B",
                treats: vec!["Hepatitis A", "Hepatitis B"],
                initial_schedule: DoseSchedule::Repeated { number: 3, interval: 6 },
                booster_schedule: BoosterSchedule::Lifetime,
                notes: "Not recommended for adults despite hepA/hepB being individually recommended. 🤷",
            }),
            // Polio (IPV) [4 doses for children; no recommendation for adults, but get a booster if you're at risk or risk averse],
            ("IPV", Vaccine {
                name: "IPV",
                treats: vec!["Polio"],
                initial_schedule: DoseSchedule::Repeated { number: 4, interval: 4 },
                booster_schedule: BoosterSchedule::Lifetime,
                notes: "No recommendation for adults, but get a booster if you're at risk or risk averse.",
            }),
            // Chickenpox [2x doses 1-6mo apart; recommended if at risk or haven't had chickenpox yet]
            ("Chickenpox", Vaccine {
                name: "Chickenpox",
                treats: vec!["Chickenpox"],
                initial_schedule: DoseSchedule::RepeatedRange { number: 2, minimum: 1, maximum: 6 },
                booster_schedule: BoosterSchedule::Lifetime,
                notes: "Recommended if at risk or haven't had chickenpox yet, but low risk so why not?",
            })]))
    }

    // TODO: allow for some shots to have happened already. Need a record struct.
    pub fn schedule(
        now: &Zoned,
        prio: impl Iterator<Item = String>,
        nshots: u8,
        end_plan_year: i16,
        _records: Vec<VaccineRecord>,
    ) -> Result<Vec<VaccineAppointment>> {
        fn add_shot_to_best_slot(
            mut mo: u32,
            nshots: u8,
            vaccine: &Vaccine,
            slots: &mut HashMap<u32, Vec<String>>,
        ) -> u32 {
            loop {
                let slot = slots.entry(mo).or_default();
                if slot.len() < nshots as usize {
                    slot.push(vaccine.name().to_owned());
                    return mo;
                } else {
                    mo = mo.checked_add(1).unwrap();
                }
            }
        }

        let day_of_mo = now
            .first_of_month()?
            .until(now)?
            .round(SpanRound::new().smallest(Unit::Day).relative(now))?
            .get_days();
        let days_left_in_month = now.days_in_month() as i32 - day_of_mo;
        let max_doses_in_mo0 = days_left_in_month * nshots as i32;
        assert!((0..400).contains(&max_doses_in_mo0));

        let vaccines = Vaccine::get_vaccines();
        // Note: track everything in months offset from now and only convert to
        //       real times with now base when we commit to an appointment.
        let mut slots = HashMap::<u32, Vec<String>>::new();
        let mut appointments = Vec::new();
        for vaccine_name in prio {
            let vaccine = vaccines.get(vaccine_name.as_str()).unwrap();
            let mut last_dose_mo = 0;
            for dose_mo in vaccine.dosage_schedule().all_months() {
                last_dose_mo = add_shot_to_best_slot(dose_mo, nshots, vaccine, &mut slots);
                appointments.push(VaccineAppointment::from_month_offset(
                    vaccine.name(),
                    now,
                    last_dose_mo,
                ));
            }
            for booster_mo in vaccine
                .booster_schedule()
                .all_months(dbg!(last_dose_mo), end_plan_year as u32 * 12)
            {
                // Start boosters after the last dose.
                let tmp_mo = add_shot_to_best_slot(booster_mo, nshots, vaccine, &mut slots);
                appointments.push(VaccineAppointment::from_month_offset(
                    vaccine.name(),
                    now,
                    tmp_mo,
                ));
            }
        }
        appointments.sort();
        Ok(appointments)
    }
}

pub enum RecordKind {
    Dose,
    Booster,
}

pub struct VaccineRecord {
    vaccine: String,
    kind: RecordKind,
    notes: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub struct VaccineAppointment {
    vaccine: String,
    year: i16,
    month: i8,
}

impl VaccineAppointment {
    pub fn vaccine(&self) -> &str {
        &self.vaccine
    }

    pub fn year(&self) -> i16 {
        self.year
    }

    pub fn month(&self) -> i8 {
        self.month
    }

    fn from_month_offset(vaccine: &str, now: &Zoned, mo: u32) -> Self {
        let (year, month) = Self::mo_to_ym(now, mo);
        VaccineAppointment {
            vaccine: vaccine.to_string(),
            year,
            month,
        }
    }

    fn mo_to_ym(now: &Zoned, mo: u32) -> (i16, i8) {
        // guaranteed to be in range 1..=12
        let month = now.date().month();
        let year = now.date().year();
        // note: move to 0-based month offsets so we can div and mod easily.
        let month_offset = month as u32 + mo - 1;
        let year_offset: i16 = (month_offset / 12).try_into().unwrap();
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
    use jiff::civil::Date;
    use jiff::tz::TimeZone;

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
    fn test_mo_for_vaccine() -> Result<()> {
        assert_eq!(
            vec![0, 6, 12],
            Vaccine::get_vaccines()
                .get("Tdap")
                .unwrap()
                .dosage_schedule()
                .all_months()
        );
        assert_eq!(
            vec![0, 1],
            Vaccine::get_vaccines()
                .get("Mpox")
                .unwrap()
                .dosage_schedule()
                .all_months()
        );
        assert_eq!(
            vec![60],
            Vaccine::get_vaccines()
                .get("Mpox")
                .unwrap()
                .booster_schedule()
                .all_months(0, 5 * 12 + 1)
        );
        Ok(())
    }
}

// pub struct ReceivedDose {
//     vaccine: String,
//     date: Date,
//     location: String,
//     notes: String,
// }
