use clap::Args;

#[derive(Args, Debug)]
pub(crate) struct StatsArgs {
    #[arg(short, long)]
    pub json: bool,
}

#[allow(clippy::items_after_test_module)]
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

pub(crate) fn run(args: StatsArgs) {
    println!("Showing stats, json: {}", args.json);
}
