use std::sync::OnceLock;

pub struct Vaccine {
    name: &'static str,
    treats: Vec<&'static str>,
    initial_schedule: DoseSchedule,
    booster_schedule: BoosterSchedule,
    notes: &'static str,
}

// Record the number of months between doses.
pub enum DoseSchedule {
    Single,
    Repeated { number: u32, interval: u32 },
    RepeatedRange { number: u32, minimum: u32, maximum: u32 },
    Todo,
}

pub enum BoosterSchedule {
    Annual,
    Years(u32),
    Lifetime,
    Todo,
}

/*
COVID-19,
Flu,

5-10 years efficacy
Tdap,

7-10 Years

Lifetime, probably? So far, herd immunity has made it impossible to research long-term efficacy. So there's another silver lining for 2025, I guess?
Pneumonia [2x doses 6mo apart; recommended for at risk and 50+, but no risk to get it sooner, so why not] (PCV20),
HPV [3x doses every 6mo for 1.5 years; lower efficacy if over 25, but why not] (Gardacil-9),
Hepatitis B [single dose; >30 years proven durability],
Hepatitis A [2 doses 6mo apart; >25 years proven durability],
Hepatitis A&B [3 doses; not recommended for adults despite hepA/hepB individually recommended 🤷],
Polio (IPV) [4 doses for children; no recommendation for adults, but get a booster if you're at risk or risk averse],
Chickenpox [2x doses 1-6mo apart; recommended if at risk or haven't had chickenpox yet]
*/

impl Vaccine {
    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn get_vaccines() -> &'static Vec<Vaccine> {
        static VACCINES: OnceLock<Vec<Vaccine>> = OnceLock::new();
        VACCINES.get_or_init(|| vec![
            // Annual
            Vaccine {
                name: "COVID-19",
                treats: vec!["COVID-19"],
                initial_schedule: DoseSchedule::Todo,
                booster_schedule: BoosterSchedule::Annual,
                notes: "Get a booster in Sept/Oct to catch any new variants."
            },
            Vaccine {
                name: "Flu",
                treats: vec!["Flu"],
                initial_schedule: DoseSchedule::Todo,
                booster_schedule: BoosterSchedule::Annual,
                notes: "Get a booster in Sept/Oct to catch any new variants."
            },
            // 5-10 years
            Vaccine {
                name: "Tdap",
                treats: vec!["Tuberculosis", "Tetanus", "Diphtheria", "Pertussis"],
                initial_schedule: DoseSchedule::Repeated { number: 3, interval: 6 },
                booster_schedule: BoosterSchedule::Todo,
                notes: "",
            },
// Mpox (M is for Monkey and Small) [2x doses 1mo apart; 5 year boosters after],
            Vaccine {
                name: "Mpox",
                treats: vec!["Monkeypox", "Smallpox"],
                initial_schedule: DoseSchedule::RepeatedRange { number: 2, minimum: 1, maximum: 6 },
                booster_schedule: BoosterSchedule::Years(5),
                notes: "M is for \"Monkey\" (and Small)",
            },
            Vaccine {
                name: "Meningitis",
                treats: vec!["Meningitis"],
                initial_schedule: DoseSchedule::Repeated { number: 2, interval: 6 },
                booster_schedule: BoosterSchedule::Years(5),
                notes: "Only recommened for adults that are exposed regularly, but low risk to get it so why not?",
            },
// MMR [2x doses 5 years apart, may need to re-dose for mumps every 5 years, if that's ever a thing again],
            Vaccine {
                name: "MMR",
                treats: vec!["Measles", "Mumps", "Rubella"],
                initial_schedule: DoseSchedule::Repeated { number: 2, interval: 5 * 12 },
                booster_schedule: BoosterSchedule::Years(5),
                notes: "Recommended for children and immuno-compromised, but again low risk so why not? Note: measles and rubella are lifetime immunity, but mumps requires a 5 year booster.",
            },
            // 7-10 years
// Shingles (Shinglex) [2x doses 2-6mo apart with 10 year boosters or closer if at risk; recommended for children and immuno-compromised, but again low risk so why not],
            Vaccine {
                name: "Shinglex",
                treats: vec!["Shingles"],
                initial_schedule: DoseSchedule::RepeatedRange { number: 2, minimum: 2, maximum: 6 },
                booster_schedule: BoosterSchedule::Years(7),
                notes: "Recommended for children and immuno-compromised, but again low risk so why not?",
            },
            // Lifetime, probably? So far, herd immunity has made it impossible to research long-term efficacy. So there's another silver lining for 2025, I guess?
// Pneumonia [2x doses 6mo apart; recommended for at risk and 50+, but no risk to get it sooner, so why not] (PCV20),
            Vaccine {
                name: "PCV20",
                treats: vec!["Pneumonia"],
                initial_schedule: DoseSchedule::Repeated { number: 2, interval: 6 },
                booster_schedule: BoosterSchedule::Lifetime,
                notes: "Recommended for at risk and 50+, but no risk to get it sooner, so why not?",
            },
// HPV [3x doses every 6mo for 1.5 years; lower efficacy if over 25, but why not] (Gardacil-9),
            Vaccine {
                name: "Gardacil-9",
                treats: vec!["Human Papillomavirus (HPV)"],
                initial_schedule: DoseSchedule::Repeated { number: 3, interval: 6 },
                booster_schedule: BoosterSchedule::Lifetime,
                notes: "",
            },
// Hepatitis B [single dose; >30 years proven durability],
            Vaccine {
                name: "Hepatitis B",
                treats: vec!["Hepatitis B"],
                initial_schedule: DoseSchedule::Single,
                booster_schedule: BoosterSchedule::Lifetime,
                notes: "Greater than 30 years proven durability. Definitely worth it.",
            },
// Hepatitis A [2 doses 6mo apart; >25 years proven durability],
            Vaccine {
                name: "Hepatitis A",
                treats: vec!["Hepatitis A"],
                initial_schedule: DoseSchedule::Repeated { number: 2, interval: 6 },
                booster_schedule: BoosterSchedule::Lifetime,
                notes: "Greater than 25 years proven durability. Definitely worth it.",
            },
// Hepatitis A&B [3 doses; not recommended for adults despite hepA/hepB individually recommended 🤷],
            Vaccine {
                name: "Hepatitis A&B",
                treats: vec!["Hepatitis A", "Hepatitis B"],
                initial_schedule: DoseSchedule::Repeated { number: 3, interval: 6 },
                booster_schedule: BoosterSchedule::Lifetime,
                notes: "Not recommended for adults despite hepA/hepB being individually recommended. 🤷",
            },
// Polio (IPV) [4 doses for children; no recommendation for adults, but get a booster if you're at risk or risk averse],
            Vaccine {
                name: "IPV",
                treats: vec!["Polio"],
                initial_schedule: DoseSchedule::Repeated { number: 4, interval: 4 },
                booster_schedule: BoosterSchedule::Lifetime,
                notes: "No recommendation for adults, but get a booster if you're at risk or risk averse.",
            },
// Chickenpox [2x doses 1-6mo apart; recommended if at risk or haven't had chickenpox yet]
            Vaccine {
                name: "Chickenpox",
                treats: vec!["Chickenpox"],
                initial_schedule: DoseSchedule::RepeatedRange { number: 2, minimum: 1, maximum: 6 },
                booster_schedule: BoosterSchedule::Lifetime,
                notes: "Recommended if at risk or haven't had chickenpox yet, but low risk so why not?",
            }])
    }
}

// pub struct ReceivedDose {
//     vaccine: String,
//     date: Date,
//     location: String,
//     notes: String,
// }

// Lifetime, probably? So far, herd immunity has made it impossible to research long-term efficacy. So there's another silver lining for 2025, I guess?
