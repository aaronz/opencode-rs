use clap::{Args, Subcommand};

#[derive(Args, Debug)]
pub(crate) struct DbArgs {
    #[command(subcommand)]
    pub action: DbAction,
}

#[derive(Subcommand, Debug)]
pub(crate) enum DbAction {
    Init,
    Migrate,
    Backup,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_db_args_init() {
        let args = DbArgs {
            action: DbAction::Init,
        };
        match args.action {
            DbAction::Init => {}
            _ => panic!("Expected Init"),
        }
    }

    #[test]
    fn test_db_args_migrate() {
        let args = DbArgs {
            action: DbAction::Migrate,
        };
        match args.action {
            DbAction::Migrate => {}
            _ => panic!("Expected Migrate"),
        }
    }

    #[test]
    fn test_db_args_backup() {
        let args = DbArgs {
            action: DbAction::Backup,
        };
        match args.action {
            DbAction::Backup => {}
            _ => panic!("Expected Backup"),
        }
    }
}

pub(crate) fn run(args: DbArgs) {
    println!("DB action: {:?}", args.action);
}
