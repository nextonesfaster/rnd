mod error;

use std::collections::HashMap;
use std::fmt::Display;
use std::hash::Hash;
use std::str::FromStr;

use clap::{Parser, Subcommand, ValueEnum};
use error::{exit, Result};
use itertools::Itertools;
use rand::distributions::uniform::SampleUniform;
use rand::distributions::{Alphanumeric, DistString, Uniform, WeightedIndex};
use rand::prelude::{Distribution, SliceRandom};
use rand::Rng;

const ABOUT: &str = "rnd lets you select random data in different ways.";
const AMOUNT_THRESHOLD: usize = 10;

const HELP_TEMPLATE: &str = r"{before-help}{bin} {version}
{author}

{about}

{usage-heading}
    {usage}

{all-args}{after-help}";

/// Represents the CLI app.
#[derive(Debug, Clone, Parser)]
#[clap(author, version, about, long_about = ABOUT)]
#[clap(help_template = HELP_TEMPLATE)]
#[clap(propagate_version = true)]
struct Cli {
    /// The subcommand.
    #[clap(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Clone, Subcommand)]
enum Command {
    /// Flip a coin `amount` times.
    #[clap(aliases = &["toss", "flip"])]
    Coin {
        /// The number of times to flip a coin.
        #[clap(default_value_t = 1)]
        amount: usize,
        /// Show the number of times heads and tails were selected.
        #[clap(short, long)]
        count: bool,
        /// Show the result of every flip in order.
        ///
        /// This is enabled by default (up to a max threshold), but using the
        /// `count` flag disables it. Explicitly passing this flag enables it even
        /// with the `count` flag.
        #[clap(short = 'A', long)]
        all: bool,
    },
    /// Choose `amount` elements from a list of items.
    ///
    /// The items can optionally have a weight and be chosen with or without repetition.
    #[clap(alias = "select")]
    Choose {
        /// The items to choose from.
        items: Vec<String>,
        /// The number of items to choose.
        #[clap(short, long, default_value_t = 1, short_alias = 'n')]
        amount: usize,
        /// The list of comma-separated weights of the items.
        ///
        /// The number of weights must be equal to the number of items.
        #[clap(short, long, use_value_delimiter = true)]
        weights: Vec<f64>,
        /// Show the number of times each item was selected.
        #[clap(short, long)]
        count: bool,
        /// Show the result of every choice.
        ///
        /// This is enabled by default (up to a max threshold), but using the
        /// `count` flag disables it. Explicitly passing this flag enables it even
        /// with the `count` flag.
        #[clap(short = 'A', long)]
        all: bool,
        /// Choose items with repetition.
        #[clap(short, long)]
        repetition: bool,
    },
    /// Shuffle a list of items.
    #[clap(alias = "shfl")]
    Shuffle {
        /// The items to shuffle.
        items: Vec<String>,
    },
    /// Print a random number between 0.0 and 1.0 (not inclusive).
    ///
    /// You can optionally provide a lower and upper bound.
    #[clap(alias = "rand")]
    Random {
        /// Include the upper bound.
        ///
        /// It is not included by default.
        #[clap(short, long)]
        inclusive: bool,
        /// The precision of a floating point number.
        #[clap(short, long, default_value_t = 6)]
        precision: usize,
        /// The lower bound of the range.
        start: Option<Num>,
        /// The upper bound of the range.
        end: Option<Num>,
    },
    /// Generates a random alphanumeric string.
    ///
    /// The default length of the string is 10 and it is lowercase.
    #[clap(alias = "str")]
    String {
        /// The length of the string.
        #[clap(short, short_alias = 'n', long, default_value_t = 10)]
        length: usize,
        /// The case of the string.
        #[clap(short, long, default_value_t = Case::Lower, value_enum)]
        case: Case,
    },
    /// Rolls a n-sided die.
    ///
    /// By default, rolls a 6-sided die.
    #[clap(alias = "dice")]
    Die {
        /// The length of the string.
        #[clap(default_value_t = 6)]
        sides: usize,
        /// The number of times to roll the die.
        #[clap(short, short_alias = 'n', long, default_value_t = 1)]
        times: usize,
        /// Show the number of times each number was rolled.
        #[clap(short, long)]
        count: bool,
        /// Show the result of every roll.
        ///
        /// This is enabled by default (up to a max threshold), but using the
        /// `count` flag disables it. Explicitly passing this flag enables it even
        /// with the `count` flag.
        #[clap(short = 'A', long)]
        all: bool,
    },
    /// Assigns items from one list to another randomly.
    ///
    /// Both lists must be of equal length.
    #[clap(alias = "assn")]
    Assign {
        /// The items on the left side of the assignment.
        #[clap(short, long, use_value_delimiter = true)]
        left: Vec<String>,
        /// The items on the right side of the assignment.
        #[clap(short, long, use_value_delimiter = true)]
        right: Vec<String>,
    },
}

impl Default for Command {
    fn default() -> Self {
        Self::Random {
            inclusive: false,
            precision: 2,
            start: Some(Num::FLOAT_0),
            end: Some(Num::FLOAT_1),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
enum Num {
    Float(f64),
    Int(i128),
}

impl Num {
    const INT_0: Num = Num::Int(0);
    const FLOAT_0: Num = Num::Float(0.0);
    const FLOAT_1: Num = Num::Float(1.0);

    fn as_float(&self) -> f64 {
        match *self {
            Self::Int(i) => i as f64,
            Self::Float(f) => f,
        }
    }
}

impl FromStr for Num {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        s.parse::<i128>()
            .map(Num::Int)
            .or_else(|_| (s.parse::<f64>().map(Num::Float)))
            .map_err(|e| e.to_string())
    }
}

impl Display for Num {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Num::Int(i) => i.fmt(fmt),
            Num::Float(f) => f.fmt(fmt),
        }
    }
}

#[derive(ValueEnum, Debug, Clone, Copy)]
enum Case {
    Lower,
    Upper,
    Mixed,
}

fn random_cmd<T: PartialOrd + SampleUniform + Display>(
    lower: T,
    upper: T,
    inclusive: bool,
    precision: usize,
) -> Result<()> {
    if lower >= upper {
        return Err("lower bound should be smaller than upper".into());
    }

    let mut rng = rand::thread_rng();

    let num = if inclusive { rng.gen_range(lower..=upper) } else { rng.gen_range(lower..upper) };

    println!("{num:.precision$}");

    Ok(())
}

fn shuffle_cmd(items: &mut [String]) {
    let mut rng = rand::thread_rng();
    items.shuffle(&mut rng);
    println!("{}", items.iter().join(", "));
}

fn choose_with_repetition<S: Display + Eq + Hash>(
    items: Vec<S>,
    weights: Vec<f64>,
    amount: usize,
    count: bool,
    all: bool,
) -> Result<()> {
    let dist = WeightedIndex::new(weights)?;
    let mut rng = rand::thread_rng();

    let selections = (0..amount).map(|_| &items[dist.sample(&mut rng)]);
    print_selections(selections, count, all, amount);

    Ok(())
}

fn choose_without_repetition(
    items: Vec<String>,
    weights: Vec<f64>,
    amount: usize,
    count: bool,
    all: bool,
) -> Result<()> {
    let choices = items
        .into_iter()
        .zip(weights.into_iter())
        .collect::<Vec<_>>();

    let selections = choices
        .choose_multiple_weighted(&mut rand::thread_rng(), amount, |i| i.1)?
        .map(|(i, _)| i);
    print_selections(selections, count, all, amount);

    Ok(())
}

fn print_selections<'a, I, D>(mut selections: I, count: bool, all: bool, amount: usize)
where
    I: Iterator<Item = &'a D>,
    D: 'a + Display + Eq + Hash,
{
    if count {
        let mut map = HashMap::new();
        for (i, selection) in selections.enumerate() {
            let entry = map.entry(selection).or_insert(0u64);
            *entry += 1;
            if all {
                print!("{}", selection);
                if i != amount - 1 {
                    print!(", ");
                }
            }
        }
        if all {
            println!("\n");
        }
        println!(
            "{}",
            map.into_iter()
                .sorted_by(|(_, a), (_, b)| b.cmp(a))
                .map(|(s, c)| format!("{s}: {c}"))
                .join("\n")
        );
    } else {
        println!("{}", selections.join(", "));
    }
}

fn string_cmd(characters: usize, case: Case) {
    let mut s = Alphanumeric.sample_string(&mut rand::thread_rng(), characters);

    match case {
        Case::Lower => s.make_ascii_lowercase(),
        Case::Upper => s.make_ascii_uppercase(),
        Case::Mixed => (),
    }

    println!("{s}");
}

fn die_cmd(sides: usize, times: usize, count: bool, all: bool) -> Result<()> {
    if sides < 1 {
        return Err("number of sides must be at least 1".into());
    }

    let mut rng = rand::thread_rng();

    let distr = Uniform::new_inclusive(1, sides);
    let roll_die = distr.sample_iter(&mut rng);

    let selections = roll_die.take(times).collect::<Vec<_>>();
    print_selections(selections.iter(), count, all, times);

    Ok(())
}

fn assign_cmd(left: &[impl Display], right: &mut [impl Display]) -> Result<()> {
    if left.len() != right.len() {
        return Err("`left` and `right` lists of unequal length".into());
    }

    let mut rng = rand::thread_rng();
    right.shuffle(&mut rng);

    println!(
        "{}",
        left.iter()
            .zip(right.iter())
            .map(|(l, r)| format!("{l}: {r}"))
            .join("\n")
    );

    Ok(())
}

fn run_cli() -> Result<()> {
    let app = Cli::parse();

    match app.command.unwrap_or_default() {
        Command::Coin {
            amount,
            count,
            all,
            ..
        } => {
            let all = all || amount <= AMOUNT_THRESHOLD;
            let count = count || !all;

            choose_with_repetition(vec!["heads", "tails"], vec![1.0, 1.0], amount, count, all)?
        },
        Command::Choose {
            amount,
            mut weights,
            items,
            count,
            all,
            repetition,
            ..
        } => {
            if weights.is_empty() {
                weights = [1.0].repeat(items.len())
            }

            let all = all || amount <= AMOUNT_THRESHOLD;
            let count = count || !all;

            if repetition || amount > items.len() {
                choose_with_repetition(items, weights, amount, count, all)?;
            } else {
                choose_without_repetition(items, weights, amount, count, all)?;
            }
        },
        Command::Shuffle {
            mut items, ..
        } => shuffle_cmd(&mut items),
        Command::Random {
            mut start,
            mut end,
            inclusive,
            precision,
            ..
        } => {
            if let (Some(s), true) = (start, end.is_none()) {
                if s.as_float() < 0.0 {
                    end = Some(Num::INT_0);
                } else {
                    end = start;
                    start = Some(Num::INT_0);
                }
            }

            match (start.unwrap_or(Num::FLOAT_0), end.unwrap_or(Num::FLOAT_1)) {
                (Num::Int(s), Num::Int(e)) => random_cmd(s, e, inclusive, precision),
                (Num::Int(s), Num::Float(e)) => random_cmd(s as f64, e, inclusive, precision),
                (Num::Float(s), Num::Int(e)) => random_cmd(s, e as f64, inclusive, precision),
                (Num::Float(s), Num::Float(e)) => random_cmd(s, e, inclusive, precision),
            }?
        },
        Command::String {
            length: characters,
            case,
            ..
        } => string_cmd(characters, case),
        Command::Die {
            sides,
            times,
            count,
            all,
            ..
        } => {
            let all = all || times <= AMOUNT_THRESHOLD;
            let count = count || !all;

            die_cmd(sides, times, count, all)?
        },
        Command::Assign {
            left,
            mut right,
        } => assign_cmd(&left, &mut right)?,
    }

    Ok(())
}

fn main() {
    if let Err(e) = run_cli() {
        exit(e, 1);
    }
}
