use std::fs::File;
use std::io::BufReader;

mod server_config_sheet;

#[derive(Debug)]
pub struct SystemReport {
    archive: zip::ZipArchive<BufReader<File>>,
    client_logs: Vec<(String, zip::DateTime)>,
    server_logs: Vec<(String, zip::DateTime)>,
    server_config_sheet: Option<server_config_sheet::Document>,
}

pub fn open(path: &str) -> anyhow::Result<SystemReport> {
    let reader = std::fs::File::open(path)?;
    let mut zip = zip::ZipArchive::new(BufReader::new(reader))?;

    let mut client_logs = Vec::new();
    let mut server_logs = Vec::new();
    let mut server_config_sheet = None;

    for i in 0..zip.len() {
        let file = zip.by_index(i)?;

        println!("Filename: {}", file.name());

        if file.name().ends_with(".log") {
            if file.name().starts_with("Server/AcsService.exe") {
                server_logs.push((file.name().to_string(), file.last_modified()));
            } else if file.name().starts_with("Client/AcsClient.exe") {
                client_logs.push((file.name().to_string(), file.last_modified()));
            }
        }

        if file.name().ends_with("ServerConfigurationSheet.xml") {
            server_config_sheet = Some(quick_xml::de::from_reader(BufReader::new(file))?);
        }
    }

    let system_report = SystemReport {
        archive: zip,
        client_logs,
        server_logs,
        server_config_sheet,
    };

    Ok(system_report)
}
