use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub(super) struct Flags {
    #[structopt(short = "c", long = "config", default_value = "config.yaml")]
    /// Path to config file
    pub config: String,

    #[structopt(long = "print-default-config")]
    /// Напечатать настройки по умолчанию и завершить выполнение.
    ///
    /// в config.yaml достаточно переопредлить настройки, которым не подходит вариант по умолчанию
    /// и обязательные настройки.
    pub print_default_config: bool,

    #[structopt(long = "print-customers-example")]
    /// Напечатать пример customers.yaml и выйти завершить выполнение.
    pub print_customers_example: bool,
}

impl Flags {
    pub fn args() -> Self {
        Flags::from_args()
    }
}
