use std::collections::HashMap;
use std::future::Future;
use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use rand::random;
use reqwest::dns::{Name, Resolve, Resolving};
use surge_ping::{Client, Config, IcmpPacket, PingIdentifier, PingSequence};
use tokio::sync::Mutex;
use tokio::task;
use tokio::time::interval;
use trust_dns_resolver::config::{NameServerConfig, Protocol, ResolverConfig, ResolverOpts};
use trust_dns_resolver::TokioAsyncResolver;

pub struct Resolver(
    TokioAsyncResolver,
    Arc<Mutex<HashMap<String, IpAddr>>>
);

impl Resolver {
    pub fn new() -> Resolver {
        Self::new_diy(&[
            IpAddr::from_str("1.1.1.1").unwrap(),
            IpAddr::from_str("8.8.8.8").unwrap(),
            IpAddr::from_str("119.29.29.29").unwrap(),
            IpAddr::from_str("1.0.0.1").unwrap(),
        ])
    }

    pub fn new_diy(dns_servers: &[IpAddr]) -> Resolver {
        assert!(!dns_servers.is_empty());
        let mut rc = ResolverConfig::new();
        for addr in dns_servers {
            rc.add_name_server(NameServerConfig {
                socket_addr: SocketAddr::new(*addr, 53),
                protocol: Protocol::Udp,
                tls_dns_name: None,
                trust_negative_responses: false,
                bind_addr: None,
            });
        }
        let mut opts = ResolverOpts::default();
        opts.use_hosts_file = false;
        opts.num_concurrent_reqs = dns_servers.len();
        Resolver(
            TokioAsyncResolver::tokio(rc, opts),
            Arc::new(Mutex::new(HashMap::new()))
        )
    }
}

async fn ping(client: Client, addr: IpAddr) -> f64 {
    let payload = [0; 56];
    let mut pinger = client.pinger(addr, PingIdentifier(random())).await;
    pinger.timeout(Duration::from_secs(1));
    let mut interval = interval(Duration::from_secs(1));
    let mut cost = 0f64;
    for idx in 0..5 {
        interval.tick().await;
        match pinger.ping(PingSequence(idx), &payload).await {
            Ok((IcmpPacket::V4(_packet), dur)) => {
                cost += dur.as_secs_f64();
            },
            Ok((IcmpPacket::V6(_packet), dur)) => {
                cost += dur.as_secs_f64();
            },
            Err(e) => {
                warn!("No.{}: {} ping {}", idx, pinger.host, e);
                return 0f64;
            },
        };
    }
    return cost;
}

impl Resolve for Resolver {
    fn resolve(&self, name: Name) -> Resolving {
        let resolver = self.0.clone();
        let dns_cache = self.1.clone();
        Box::pin(async move {
            let mut dns_cache = dns_cache.lock().await;
            if let Some(addr) = dns_cache.get(name.as_str()) {
                return Ok(Box::new(vec![
                    SocketAddr::new(*addr, 0)
                ].into_iter()) as Box<_>);
            }

            let ips = resolver.lookup_ip(name.as_str()).await?;
            let addrs = ips
                .into_iter()
                .map(|ip| ip);

            info!("Preferred IP started, domain: {}", name.as_str());

            let client_v4 = Client::new(&Config::default()).unwrap();
            let mut min_cost = 1000000.0f64;
            let mut suggested_ip: Option<IpAddr> = None;
            for ip in addrs {
                let cost = ping(client_v4.clone(), ip).await;
                if cost != 0f64 && cost < min_cost {
                    min_cost = cost;
                    suggested_ip = Some(ip);
                }
            };
            if suggested_ip.is_none() {
                return Err("No available IP".into());
            }

            info!("Preferred IP completed: {} => {:?}, time: {:0.2?}s", name.as_str(), suggested_ip, min_cost / 5.0);

            dns_cache.insert(name.as_str().to_string(), suggested_ip.unwrap());

            Ok(Box::new(vec![SocketAddr::new(suggested_ip.unwrap(), 0)].into_iter()) as Box<_>)
        })
    }
}