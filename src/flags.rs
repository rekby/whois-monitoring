use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub(super) struct Flags {
    #[structopt(short = "c", long = "config", default_value = "config.yaml")]
    /// Path to config file
    pub config: String,

    #[structopt(long = "print-default-config")]
    /// It need write to config.yaml only required field and options, which you want override.
    /// No need copy default opetions values.
    pub print_default_config: bool,

    #[structopt(long = "print-customers-example")]
    /// Print customers.yaml example and exit.
    pub print_customers_example: bool,
}

impl Flags {
    pub fn args() -> Self {
        Flags::from_args()
    }
}
