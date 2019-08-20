mod account_checker;
mod cache;
mod config;
mod customers_config;
mod errors;
mod flags;

use {
    crate::{
        account_checker::{AccountChecker, CheckAccountResult},
        config::Config,
        customers_config::CustomerConfig,
        errors::Result,
    },
    chrono::{DateTime, Datelike, Utc},
    lettre::Transport,
    rand,
    rand::prelude::*,
    slog::{debug, error, info, o, Drain, Level},
    slog_unwraps::ResultExt,
    std::collections::HashMap,
    std::fs,
    std::io,
};

const CUSTOMERS_EXAMPLE_YAML: &str = include_str!("../customers-example.yaml");

fn get_config(fname: &str) -> Result<Config> {
    return Config::from_file(fname);
}
fn get_customers(fname: &str) -> Result<Vec<CustomerConfig>> {
    let f = fs::File::open(fname)?;
    let config = serde_yaml::from_reader(f)?;
    return Ok(config);
}

fn is_need_send(
    cfg: &Config,
    customer: &customers_config::CustomerConfig,
    acc_result: &CheckAccountResult,
    now: &chrono::DateTime<Utc>,
) -> bool {
    if customer.domains.iter().all(|item| item.disabled) {
        return false;
    };

    if cfg.ok_report_day == 0 {
        return true;
    };

    if account_checker::need_attention(cfg, acc_result, now) {
        return true;
    }

    return cfg.ok_report_day - 1 == now.weekday() as u8;
}

enum CreateEmailParams {
    ToCustomer,
    ToAdmin,
}

fn create_email(
    params: CreateEmailParams,
    from: &str,
    customer: &CustomerConfig,
    check_result: &CheckAccountResult,
) -> lettre_email::EmailBuilder {
    let subject = match params {
        CreateEmailParams::ToAdmin => format!("Отчет по доменам - {}", customer.name),
        CreateEmailParams::ToCustomer => "Отчет по доменам".to_string(),
    };
    let report_text = account_checker::create_account_report(check_result);
    let res = lettre_email::Email::builder()
        .subject(subject)
        .from(from)
        .alternative(format!("<pre>\n{}\n</pre>", report_text), &report_text);

    return res;
}

fn create_logger(cfg: &Config) -> slog::Logger {
    let decorator = slog_term::TermDecorator::new().build();
    let drain = match cfg.log_format {
        config::LogFormat::Lines => {
            let drain = slog_term::FullFormat::new(decorator).build().fuse();
            slog_async::Async::new(drain).build().fuse()
        }
        config::LogFormat::Hierarchy => {
            let drain = slog_term::CompactFormat::new(decorator).build().fuse();
            slog_async::Async::new(drain).build().fuse()
        }
    };
    let level = match cfg.log_level {
        config::LogLevel::Debug => Level::Debug,
        config::LogLevel::Info => Level::Info,
        config::LogLevel::Error => Level::Error,
    };
    let drain = drain.filter_level(level).fuse();
    let start_id: u64 = random();
    let log = slog::Logger::root(
        drain,
        o!(
        "start-id"=>start_id
        ),
    );
    return log;
}

fn send_email(log: &slog::Logger, cfg: &Config, email: lettre_email::EmailBuilder) -> Result<()> {
    use lettre::smtp::authentication::Credentials;
    use lettre::SmtpClient;
    let mut smtp_client = SmtpClient::new_simple(cfg.smtp_server.as_str())?
        .credentials(Credentials::new(
            cfg.smtp_login.clone(),
            cfg.smtp_password.clone(),
        ))
        .transport();
    let res = smtp_client
        .send(email.build().log(log, Level::Critical)?.into())
        .log(log, Level::Error)?;
    info!(log, "Email sent"; "code"=>format!("{:?}", res.code), "res-message"=>format!("{:?}", res.message));
    return Ok(());
}

fn main() -> Result<()> {
    let now = chrono::Utc::now();

    let opt = flags::Flags::args();

    if opt.print_default_config {
        println!("{}", config::DEFAULT_CONFIG_YAML);
        return Ok(());
    }

    if opt.print_customers_example {
        println!("{}", CUSTOMERS_EXAMPLE_YAML);
        return Ok(());
    }

    let cfg = get_config(&opt.config)?;

    let log = &create_logger(&cfg);

    let mut checker = AccountChecker::new();

    if cfg.state_file.is_empty() {
        debug!(log, "State file path is empty. Doesn't load state.")
    } else {
        info!(log, "Load state"; "file"=>&cfg.state_file);
        match fs::File::open(&cfg.state_file) {
            Err(err) => {
                if err.kind() == io::ErrorKind::NotFound {
                    info!(log, "State file not found.")
                } else {
                    error!(log, "State file error. You can remove it for reset cache.");
                    return Err(errors::Error::from(err));
                }
            }
            Ok(reader) => {
                // Ignore cache error
                let _ = checker
                    .load_state(log, reader, now, cfg.no_cache_days_before_expire)
                    .log(log, Level::Error);
            }
        }
    };

    let customers = match get_customers(&cfg.customers_file) {
        Err(err) => {
            error!(log,
                "Error while load costomers config";
                "file"=>&cfg.customers_file, "error"=>err.to_string()
            );
            return Err(err);
        }
        Ok(customers) => {
            debug!(log, "Customers load");
            customers
        }
    };

    run(&now, &cfg, &log, &mut checker, &customers);

    if !cfg.state_file.is_empty() {
        let writer = fs::File::create(&cfg.state_file)?;
        checker.save_state(writer)?
    }

    return Ok(());
}

fn run(
    now: &DateTime<Utc>,
    cfg: &Config,
    log: &slog::Logger,
    checker: &mut AccountChecker,
    customers: &Vec<CustomerConfig>,
) {
    let mut h = HashMap::new();
    for account in customers {
        if account.disabled {
            continue;
        }
        h.insert(account, checker.check_account(log, &account));
    }
    for (customer, check_result) in h.iter() {
        let log = &log.new(o!("customer"=>customer.name.clone()));

        if is_need_send(&cfg, *customer, check_result, &now) {
            debug!(log, "Need send report");
            let admin_email = create_email(
                CreateEmailParams::ToAdmin,
                cfg.smtp_from.as_str(),
                *customer,
                check_result,
            );
            for to in &cfg.admin_emails {
                let log = &log.new(o!("dest"=>"admin", "email"=>to.clone()));
                let _ = send_email(log, &cfg, admin_email.clone().to(to.as_str()));
            }
            let customer_email = create_email(
                CreateEmailParams::ToCustomer,
                cfg.smtp_from.as_str(),
                *customer,
                check_result,
            );
            for to in &customer.emails {
                if to.clone().to_lowercase().starts_with("off:") {
                    continue;
                }
                let log = &log.new(o!("dest"=>"customer", "email"=>to.clone()));
                let _ = send_email(log, &cfg, customer_email.clone().to(to.as_str()));
            }
        } else {
            debug!(log, "No need send record");
        }
    }
}
