use std::env;
use std::fs::{self, OpenOptions};
use std::io::{self, Read, Write};
use std::path::Path;
use std::process::ExitCode;

const MAX_COMMAND_BYTES: usize = 2_048;
const MAX_PAYLOAD_HEX_BYTES: usize = 1_024;
const MAX_RECORDS: usize = 10_000;

#[derive(Clone, Debug, Eq, PartialEq)]
struct Record {
    sequence: u64,
    payload_hex: String,
}

fn valid_hex(value: &str) -> bool {
    !value.is_empty() && value.len().is_multiple_of(2) && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn load_journal(path: &Path) -> io::Result<Vec<Record>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = fs::read_to_string(path)?;
    let mut records = Vec::new();
    for (index, line) in content.lines().enumerate() {
        let (sequence, payload_hex) = line.split_once('\t').ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidData, "malformed journal record")
        })?;
        let sequence = sequence
            .parse::<u64>()
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "invalid journal sequence"))?;
        if sequence != index as u64 + 1 || !valid_hex(payload_hex) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "non-contiguous sequence or invalid payload encoding",
            ));
        }
        records.push(Record {
            sequence,
            payload_hex: payload_hex.to_ascii_lowercase(),
        });
    }
    if records.len() > MAX_RECORDS {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "journal exceeds harness record limit",
        ));
    }
    Ok(records)
}

// Reads at most MAX_COMMAND_BYTES into memory. An oversized frame is drained
// through its newline and rejected without retaining the remainder.
fn read_bounded_line<R: Read>(reader: &mut R, buffer: &mut Vec<u8>) -> io::Result<Option<bool>> {
    buffer.clear();
    let mut byte = [0_u8; 1];
    loop {
        match reader.read(&mut byte)? {
            0 if buffer.is_empty() => return Ok(None),
            0 => return Ok(Some(true)),
            _ if byte[0] == b'\n' => return Ok(Some(true)),
            _ if buffer.len() == MAX_COMMAND_BYTES => {
                while reader.read(&mut byte)? != 0 && byte[0] != b'\n' {}
                buffer.clear();
                return Ok(Some(false));
            }
            _ => buffer.push(byte[0]),
        }
    }
}

fn run_worker(id: &str, path: &Path) -> io::Result<()> {
    let mut records = load_journal(path)?;
    let mut journal = OpenOptions::new().create(true).append(true).open(path)?;
    let stdin = io::stdin();
    let mut input = stdin.lock();
    let stdout = io::stdout();
    let mut output = stdout.lock();
    writeln!(output, "READY {id}")?;
    output.flush()?;

    let mut frame = Vec::with_capacity(MAX_COMMAND_BYTES);
    while let Some(valid_frame) = read_bounded_line(&mut input, &mut frame)? {
        if !valid_frame {
            writeln!(output, "ERR_TOO_LARGE")?;
            output.flush()?;
            continue;
        }
        let line = match std::str::from_utf8(&frame) {
            Ok(line) => line.trim_end_matches('\r'),
            Err(_) => {
                writeln!(output, "ERR_ENCODING")?;
                output.flush()?;
                continue;
            }
        };
        let fields: Vec<&str> = line.split_whitespace().collect();
        let response = match fields.as_slice() {
            ["STATUS"] => {
                let last = records.last().map_or(0, |record| record.sequence);
                format!("STATUS {} {last}", records.len())
            }
            ["APPEND", sequence, payload] => {
                let parsed = sequence.parse::<u64>();
                match parsed {
                    Err(_) => "ERR_SEQUENCE".to_owned(),
                    Ok(_) if !valid_hex(payload) => "ERR_PAYLOAD_ENCODING".to_owned(),
                    Ok(_) if payload.len() > MAX_PAYLOAD_HEX_BYTES => {
                        "ERR_PAYLOAD_TOO_LARGE".to_owned()
                    }
                    Ok(0) => "ERR_SEQUENCE".to_owned(),
                    Ok(sequence) => {
                        let payload = payload.to_ascii_lowercase();
                        if sequence <= records.len() as u64 {
                            let existing = &records[(sequence - 1) as usize].payload_hex;
                            if existing == &payload {
                                "EXISTING".to_owned()
                            } else {
                                "CONFLICT".to_owned()
                            }
                        } else if sequence != records.len() as u64 + 1 {
                            "GAP".to_owned()
                        } else if records.len() >= MAX_RECORDS {
                            "ERR_CAPACITY".to_owned()
                        } else {
                            writeln!(journal, "{sequence}\t{payload}")?;
                            journal.sync_all()?;
                            records.push(Record {
                                sequence,
                                payload_hex: payload,
                            });
                            "APPENDED".to_owned()
                        }
                    }
                }
            }
            ["SHUTDOWN"] => {
                writeln!(output, "BYE")?;
                output.flush()?;
                return Ok(());
            }
            _ => "ERR_COMMAND".to_owned(),
        };
        writeln!(output, "{response}")?;
        output.flush()?;
    }
    Ok(())
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    if args.get(1).map(String::as_str) != Some("--worker") || args.len() != 4 {
        eprintln!("usage: apex-consensus-harness --worker <id> <journal-path>");
        return ExitCode::from(2);
    }
    match run_worker(&args[2], Path::new(&args[3])) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("worker failure: {error}");
            ExitCode::FAILURE
        }
    }
}
