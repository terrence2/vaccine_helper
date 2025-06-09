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
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum BoosterSchedule {
    Annual,
    Years(u32),
    Lifetime,
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

    pub fn treats_str(&self) -> String {
        self.treats.join(", ")
    }

    pub fn all_doses(&self, end_plan_mo: u32) -> Vec<u32> {
        let mut initial = self.initial_schedule.all_months();
        assert!(!initial.is_empty(), "a vaccine has no schedule");
        let booster = self
            .booster_schedule
            .all_months(*initial.last().unwrap_or(&0), end_plan_mo);
        initial.extend(booster);
        initial
    }

    pub fn get_vaccines() -> &'static HashMap<&'static str, Vaccine> {
        static VACCINES: OnceLock<HashMap<&'static str, Vaccine>> = OnceLock::new();
        VACCINES.get_or_init(|| HashMap::from_iter([
            ("COVID-19", Vaccine {
                name: "COVID-19",
                treats: vec!["COVID-19"],
                initial_schedule: DoseSchedule::RepeatedRange { number: 2, minimum: 1, maximum: 2 },
                booster_schedule: BoosterSchedule::Annual,
                notes: "Get a booster in Sept/Oct to catch any new variants."
            }),
            ("Flu", Vaccine {
                name: "Flu",
                treats: vec!["Flu"],
                initial_schedule: DoseSchedule::Single,
                booster_schedule: BoosterSchedule::Annual,
                notes: "Get a booster in Sept/Oct to catch any new variants. Get a second dose in the middle of the season if you have no prior exposure."
            }),
            ("Tdap", Vaccine {
                name: "Tdap",
                treats: vec!["Tuberculosis", "Tetanus", "Diphtheria", "Pertussis"],
                initial_schedule: DoseSchedule::Repeated { number: 3, interval: 6 },
                booster_schedule: BoosterSchedule::Years(10),
                notes: "Tuberculosis is humanity's greatest adversary; please do your part by getting vaccinated and staying up to date with boosters!",
            }),
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
            ("MMR", Vaccine {
                name: "MMR",
                treats: vec!["Measles", "Mumps", "Rubella"],
                initial_schedule: DoseSchedule::Repeated { number: 2, interval: 5 * 12 },
                booster_schedule: BoosterSchedule::Years(5),
                notes: "Recommended for children and immuno-compromised, but again low risk so why not? Note: measles and rubella are lifetime immunity, but mumps requires a 5 year booster.",
            }),
            ("Shinglex", Vaccine {
                name: "Shinglex",
                treats: vec!["Shingles"],
                initial_schedule: DoseSchedule::RepeatedRange { number: 2, minimum: 2, maximum: 6 },
                booster_schedule: BoosterSchedule::Years(7),
                notes: "Recommended for children and immuno-compromised, but again low risk so why not?",
            }),
            ("PCV20", Vaccine {
                name: "PCV20",
                treats: vec!["Pneumonia"],
                initial_schedule: DoseSchedule::Repeated { number: 2, interval: 6 },
                booster_schedule: BoosterSchedule::Lifetime,
                notes: "Recommended for at risk and 50+, but no risk to get it sooner, so why not?",
            }),
            ("Gardacil-9", Vaccine {
                name: "Gardacil-9",
                treats: vec!["Human Papillomavirus (HPV)"],
                initial_schedule: DoseSchedule::Repeated { number: 3, interval: 6 },
                booster_schedule: BoosterSchedule::Lifetime,
                notes: "",
            }),
            ("Hepatitis B", Vaccine {
                name: "Hepatitis B",
                treats: vec!["Hepatitis B"],
                initial_schedule: DoseSchedule::Single,
                booster_schedule: BoosterSchedule::Lifetime,
                notes: "Greater than 30 years proven durability. Definitely worth it.",
            }),
            ("Hepatitis A", Vaccine {
                name: "Hepatitis A",
                treats: vec!["Hepatitis A"],
                initial_schedule: DoseSchedule::Repeated { number: 2, interval: 6 },
                booster_schedule: BoosterSchedule::Lifetime,
                notes: "Greater than 25 years proven durability. Definitely worth it.",
            }),
            ("Hepatitis A&B", Vaccine {
                name: "Hepatitis A&B",
                treats: vec!["Hepatitis A", "Hepatitis B"],
                initial_schedule: DoseSchedule::Repeated { number: 3, interval: 6 },
                booster_schedule: BoosterSchedule::Lifetime,
                notes: "Not recommended for adults despite hepA/hepB being individually recommended. 🤷",
            }),
            ("IPV", Vaccine {
                name: "IPV",
                treats: vec!["Polio"],
                initial_schedule: DoseSchedule::Repeated { number: 4, interval: 4 },
                booster_schedule: BoosterSchedule::Lifetime,
                notes: "No recommendation for adults, but get a booster if you're at risk or risk averse.",
            }),
            ("Chickenpox", Vaccine {
                name: "Chickenpox",
                treats: vec!["Chickenpox"],
                initial_schedule: DoseSchedule::RepeatedRange { number: 2, minimum: 1, maximum: 6 },
                booster_schedule: BoosterSchedule::Lifetime,
                notes: "Recommended if at risk or haven't had chickenpox yet, but low risk so why not?",
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
        nshots: u8,
        end_plan_year: i16,
        _records: Vec<VaccineRecord>,
    ) -> Result<Vec<VaccineAppointment>> {
        // Compute mo offset from current to end schedule at.
        let current_year = now.year();
        let limit_mo = (end_plan_year - current_year) as u32 * 12;

        // In general, a person can get a whole lot of vaccines in a month, even
        let day_of_mo = now
            .first_of_month()?
            .until(now)?
            .round(SpanRound::new().smallest(Unit::Day).relative(now))?
            .get_days();
        let days_left_in_month = now.days_in_month() as i32 - day_of_mo;
        let max_doses_in_mo0 = ((days_left_in_month * nshots as i32) as f32 / 7.).ceil() as u32;
        assert!((0..400).contains(&max_doses_in_mo0));
        let mut doses_in_mo0 = 0;

        let vaccines = Vaccine::get_vaccines();
        let mut appointments = Vec::new();
        for vaccine_name in prio {
            let vaccine = vaccines.get(vaccine_name.as_str()).unwrap();
            for mut dose_mo in vaccine.all_doses(limit_mo) {
                if doses_in_mo0 >= max_doses_in_mo0 {
                    dose_mo += 1;
                } else if dose_mo == 0 {
                    doses_in_mo0 += 1;
                }
                appointments.push(VaccineAppointment::from_month_offset(
                    vaccine.name(),
                    now,
                    dose_mo,
                ))
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
