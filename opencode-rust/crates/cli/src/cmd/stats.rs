use clap::Args;

#[derive(Args, Debug)]
pub struct StatsArgs {
    #[arg(short, long)]
    pub json: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stats_args_default() {
        let args = StatsArgs { json: false };
        assert!(!args.json);
    }

    #[test]
    fn test_stats_args_with_json() {
        let args = StatsArgs { json: true };
        assert!(args.json);
    }
}

pub fn run(args: StatsArgs) {
    println!("Showing stats, json: {}", args.json);
}
