use {
    crate::{
        cache, config,
        customers_config::{CustomerConfig, DomainConfig},
        errors::Result,
    },
    chrono::{DateTime, Utc},
    slog::{debug, info, o, Level},
    slog_unwraps::ResultExt,
    std::{cmp, collections::HashMap, fmt::Display, io},
};

pub(crate) struct AccountChecker {
    whois_client: whois2::Client,
    cache: cache::Cache,
}

impl AccountChecker {
    pub(crate) fn new() -> AccountChecker {
        return AccountChecker {
            whois_client: whois2::Client::new(),
            cache: cache::Cache::new(),
        };
    }

    pub(crate) fn check_account<'a>(
        &mut self,
        log: &slog::Logger,
        cust: &'a CustomerConfig,
    ) -> CheckAccountResult<'a> {
        let log = &log.new(o!("account"=>cust.name.clone()));
        let mut res = CheckAccountResult::new();
        if cust.disabled {
            info!(log, "Account disabled. Skip check.");
            return res;
        }
        for domain in &cust.domains {
            res.domain_results
                .insert(domain, self.check_domain(log, domain));
        }
        return res;
    }

    fn check_domain(
        &mut self,
        log: &slog::Logger,
        domain: &DomainConfig,
    ) -> Result<CheckDomainResult> {
        let log = &log.new(o!("domain"=>domain.domain.clone()));
        if domain.disabled {
            info!(log, "Domain disabled. Skip check.");
            return Ok(CheckDomainResult::Disabled);
        }
        debug!(log, "Start check");
        if self.cache.domains_expire.contains_key(&domain.domain) {
            debug!(log, "Read date of expire from cache");
        } else {
            info!(log, "Get expire date from whois servers");
            let whois = self
                .whois_client
                .get_whois_kv(&domain.domain)
                .log(log, Level::Error)?;
            let expire_date = get_paid_till_date(&whois).log(log, Level::Error)?;
            self.cache
                .domains_expire
                .insert(domain.domain.clone(), expire_date);
        }
        let expire_date = self.cache.domains_expire[&domain.domain];
        debug!(log, "Expire_date"; "expire"=>expire_date.to_string());
        return Ok(CheckDomainResult::ExpireDate(expire_date));
    }

    pub(crate) fn save_state<W: io::Write>(&self, writer: W) -> Result<()> {
        Ok(serde_yaml::to_writer(writer, &self.cache)?)
    }

    pub(crate) fn load_state<R: io::Read>(
        &mut self,
        log: &slog::Logger,
        reader: R,
        now: chrono::DateTime<Utc>,
        no_cache_days_before_expire: i64,
    ) -> Result<()> {
        self.cache = serde_yaml::from_reader(reader)?;
        self.cache.clean(&now, no_cache_days_before_expire);
        info!(log, "Load cache"; "domains-count"=>self.cache.domains_expire.len());
        return Ok(());
    }
}

pub(crate) struct CheckAccountResult<'a> {
    domain_results: HashMap<&'a DomainConfig, Result<CheckDomainResult>>,
}

impl<'a> CheckAccountResult<'a> {
    fn new() -> Self {
        return CheckAccountResult {
            domain_results: HashMap::new(),
        };
    }
}

enum CheckDomainResult {
    ExpireDate(chrono::DateTime<Utc>),
    Disabled,
}

impl Display for CheckDomainResult {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        use CheckDomainResult::*;
        match self {
            ExpireDate(paid_time) => f.write_str(paid_time.to_rfc3339().as_str()),
            Disabled => f.write_str("Disabled"),
        }
    }
}

pub(crate) fn create_account_report(customer_result: &CheckAccountResult) -> String {
    use CheckDomainResult::*;

    let mut domains: Vec<&DomainConfig> = customer_result
        .domain_results
        .keys()
        .map(|item| *item)
        .collect();
    domains.sort_unstable_by(|d1, d2| {
        let res1 = &customer_result.domain_results[d1];
        let res2 = &customer_result.domain_results[d2];
        match (res1, res2) {
            (Err(_), Ok(_)) => cmp::Ordering::Less,
            (Ok(_), Err(_)) => cmp::Ordering::Greater,
            (Err(_), Err(_)) => cmp::Ordering::Equal,
            (Ok(res1), Ok(res2)) => match (res1, res2) {
                (Disabled, Disabled) => d1.domain.cmp(&d2.domain),
                (Disabled, ExpireDate(_)) => cmp::Ordering::Greater,
                (ExpireDate(_), Disabled) => cmp::Ordering::Less,
                (ExpireDate(date1), ExpireDate(date2)) => date1.cmp(&date2),
            },
        }
    });
    let mut table = vec![];
    for domain_config in domains {
        let expired_column = match &customer_result.domain_results[domain_config] {
            Err(err) => format!("{}", err),
            Ok(res) => format!("{}", res),
        };
        table.push([
            domain_config.domain.clone(),
            domain_config.account.clone(),
            expired_column.clone(),
            domain_config.autorenew.to_string(),
        ])
    }

    let mut domain_column = ascii_table::ColumnConfig::default();
    domain_column.header = "Domain".to_string();

    let mut domain_account_column = ascii_table::ColumnConfig::default();
    domain_account_column.header = "Account".to_string();

    let mut expire_column = ascii_table::ColumnConfig::default();
    expire_column.header = "Expired".to_string();

    let mut autorenew_column = ascii_table::ColumnConfig::default();
    autorenew_column.header = "Autorenew".to_string();

    let mut table_config = ascii_table::TableConfig::default();
    table_config.width = 140;
    table_config.columns.insert(0, domain_column);
    table_config.columns.insert(1, domain_account_column);
    table_config.columns.insert(2, expire_column);
    table_config.columns.insert(3, autorenew_column);
    return ascii_table::format_table(&table, &table_config);
}

fn get_paid_till_date(whois: &HashMap<String, String>) -> Result<DateTime<Utc>> {
    for key in &vec!["paid-till", "registry expiry date"] {
        if let Some(date) = whois.get(*key) {
            return Ok(DateTime::parse_from_rfc3339(date)?.with_timezone(&chrono::Utc));
        }
    }
    return Err(crate::errors::Error::CanFindWhoisField);
}

pub(crate) fn need_attention(
    cfg: &config::Config,
    acc_result: &CheckAccountResult,
    now: &chrono::DateTime<Utc>,
) -> bool {
    return acc_result.domain_results.values().any(|item| match item {
        Err(_) => true,
        Ok(res) => match res {
            CheckDomainResult::Disabled => false,
            CheckDomainResult::ExpireDate(expire) => {
                let a: chrono::Duration = *expire - *now;
                return a.num_days() <= cfg.expire_soon_days as i64;
            }
        },
    });
}
