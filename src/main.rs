mod error;

use std::collections::HashMap;
use std::fmt::Display;
use std::hash::Hash;
use std::str::FromStr;

use clap::{Parser, Subcommand};
use error::{exit, Result};
use itertools::Itertools;
use rand::distributions::uniform::SampleUniform;
use rand::distributions::WeightedIndex;
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
        /// This is enabled by default up to a threshold, but using the `count` flag disables
        /// it. Explicitly passing this flag enables it even with the `count` flag.
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
        /// This is enabled by default, but using the `count` flag disables
        /// it. Explicitly passing this flag enables it even with the `count` flag.
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
}

#[derive(Debug, Clone)]
enum Num {
    Float(f64),
    Int(i128),
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

fn shuffle_cmd(mut items: Vec<String>) {
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

    print_selections(count, (0..amount).map(|_| &items[dist.sample(&mut rng)]), all, amount);

    Ok(())
}

fn choose_without_repetition(
    items: Vec<String>,
    weights: Vec<f64>,
    amount: usize,
    count: bool,
    all: bool,
) -> Result<()> {
    let choices = items.into_iter().zip(weights.into_iter()).collect::<Vec<_>>();

    print_selections(
        count,
        choices.choose_multiple_weighted(&mut rand::thread_rng(), amount, |i| i.1)?.map(|(i, _)| i),
        all,
        amount,
    );

    Ok(())
}

fn print_selections<'a, I, D>(count: bool, mut selections: I, all: bool, amount: usize)
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

fn run_cli() -> Result<()> {
    let app = Cli::parse();

    match app.command.unwrap_or(Command::Random {
        inclusive: false,
        precision: 2,
        start: Some(Num::Float(0.0)),
        end: Some(Num::Float(1.0)),
    }) {
        Command::Coin {
            amount,
            count,
            all,
            ..
        } => {
            let all = all || amount < AMOUNT_THRESHOLD;
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

            let all = all || amount < AMOUNT_THRESHOLD;
            let count = count || !all;

            if repetition || amount > items.len() {
                choose_with_repetition(items, weights, amount, count, all)?;
            } else {
                choose_without_repetition(items, weights, amount, count, all)?;
            }
        },
        Command::Shuffle {
            items, ..
        } => shuffle_cmd(items),
        Command::Random {
            mut start,
            mut end,
            inclusive,
            precision,
            ..
        } => {
            if start.is_some() && end.is_none() {
                end = start;
                start = Some(Num::Int(0));
            }

            match (start.unwrap_or(Num::Float(0.0)), end.unwrap_or(Num::Float(1.0))) {
                (Num::Int(s), Num::Int(e)) => random_cmd(s, e, inclusive, precision),
                (Num::Int(s), Num::Float(e)) => random_cmd(s as f64, e, inclusive, precision),
                (Num::Float(s), Num::Int(e)) => random_cmd(s, e as f64, inclusive, precision),
                (Num::Float(s), Num::Float(e)) => random_cmd(s, e, inclusive, precision),
            }?
        },
    }

    Ok(())
}

fn main() {
    if let Err(e) = run_cli() {
        exit(e, 1);
    }
}
