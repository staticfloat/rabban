use clap::Parser;
use csv;
use serde::Serialize;
use std::collections::HashSet;
use std::fs::canonicalize;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use sysinfo::{CpuExt, DiskExt, System, SystemExt};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Amount of time to sleep between system samplings, in seconds
    #[arg(short = 't', long, default_value_t = 10.0)]
    sleep_time: f64,

    /// List of paths that represent disks we want to track disk usage of
    #[arg(short = 'd', long)]
    disk_paths: Vec<String>,

    /// Output file in CSV format
    #[arg()]
    output_file: String,
}

/// Each one of these structs represents a snapshot of our resource usage
#[derive(Serialize, Debug)]
struct ResSnapshot {
    // TODO: Add per-core statistics, instead of just the mean of all cores
    /// CPU usage, as a percentage
    cpu_used: f64,

    /// Memory is in bytes
    mem_used: u64,
    mem_total: u64,

    /// Disk usage is in bytes
    disk_used: u64,
    disk_total: u64,

    /// UTC Timestamp, as seconds since the unix epoch
    timestamp: f64,
}

fn get_timestamp() -> f64 {
    return match SystemTime::now().duration_since(UNIX_EPOCH) {
        Err(_) => 0,
        Ok(elapsed) => elapsed.as_secs_f64(),
    };
}

fn get_mean_cpu_usage(sys: &System) -> f64 {
    let cpu_usages = sys.cpus().iter().map(|cpu| cpu.cpu_usage() as f64);
    let num_cpus = cpu_usages.len() as f64;
    return cpu_usages.sum::<f64>() / num_cpus;
}

fn collect_stats(sys: &System, disk_mountpoints: &HashSet<String>) -> ResSnapshot {
    let mut disk_total: u64 = 0;
    let mut disk_used: u64 = 0;
    for disk in sys.disks() {
        if disk_mountpoints.contains(disk.mount_point().to_str().unwrap_or("")) {
            disk_used += disk.total_space() - disk.available_space();
            disk_total += disk.total_space();
        }
    }

    // Return them in a convenient, serializeable form
    return ResSnapshot {
        cpu_used: get_mean_cpu_usage(sys),
        mem_used: sys.available_memory(),
        mem_total: sys.total_memory(),
        disk_used: disk_used,
        disk_total: disk_total,
        timestamp: get_timestamp(),
    };
}

fn main() -> () {
    let args = Args::parse();
    let sleep_time = std::time::Duration::from_secs_f64(args.sleep_time);

    // Immediately canonicalize all disk paths:
    let disk_paths: Vec<PathBuf> = args
        .disk_paths
        .iter()
        .map(|p| canonicalize(p))
        .filter(|x| x.is_ok())
        .map(|x| x.unwrap())
        .collect();

    // Open our output file
    let mut writer = csv::Writer::from_path(args.output_file).expect("Bad CSV path!");

    // Collect initial system statistics
    let mut sys = System::new_all();

    // Identify disks that are superpaths of the requested disk paths,
    // so that we can track their usage statistics over time:
    let mut disk_mountpoints = HashSet::new();
    for p in disk_paths {
        let mut best_len = 0;
        let mut best_disk = None;
        for disk in sys.disks() {
            let mp = disk.mount_point();
            let mp_len = match mp.to_str() {
                Some(s) => s.len(),
                None => 0,
            };
            if mp_len > best_len && p.starts_with(mp) {
                best_len = mp_len;
                best_disk = Some(disk);
            }
        }
        match best_disk {
            Some(disk) => {
                disk_mountpoints.insert(disk.mount_point().clone().to_str().unwrap().to_owned());
            }
            None => {
                println!("WARNING: No filesystem found for path {:?}", p);
            }
        }
    }

    loop {
        std::thread::sleep(sleep_time);

        // Refresh all of our stats
        sys.refresh_cpu();
        sys.refresh_memory();
        sys.refresh_disks();

        let snapshot = collect_stats(&sys, &disk_mountpoints);

        // Ignore writing errors, just keep on trying
        writer.serialize(snapshot).ok();
        writer.flush().ok();
    }
}
