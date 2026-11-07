use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_DIR: AtomicUsize = AtomicUsize::new(0);
const WORKER: &str = env!("CARGO_BIN_EXE_apex-consensus-harness");

struct TestDir(PathBuf);

impl TestDir {
    fn new(label: &str) -> Self {
        let serial = NEXT_DIR.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "apex-consensus-harness-{}-{label}-{serial}",
            std::process::id()
        ));
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

struct Worker {
    child: Child,
    input: ChildStdin,
    output: BufReader<std::process::ChildStdout>,
}

impl Worker {
    fn start(id: usize, journal: &Path) -> Self {
        let mut child = Command::new(WORKER)
            .arg("--worker")
            .arg(id.to_string())
            .arg(journal)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .unwrap();
        let input = child.stdin.take().unwrap();
        let output = BufReader::new(child.stdout.take().unwrap());
        let mut worker = Self { child, input, output };
        assert_eq!(worker.read_response(), format!("READY {id}"));
        worker
    }

    fn command(&mut self, command: &str) -> String {
        self.input.write_all(command.as_bytes()).unwrap();
        self.input.write_all(b"\n").unwrap();
        self.input.flush().unwrap();
        self.read_response()
    }

    fn read_response(&mut self) -> String {
        let mut line = String::new();
        self.output.read_line(&mut line).unwrap();
        assert!(!line.is_empty(), "worker exited without a response");
        line.trim_end().to_owned()
    }

    fn stop(mut self) {
        assert_eq!(self.command("SHUTDOWN"), "BYE");
        self.child.wait().unwrap();
    }
}

fn append(worker: &mut Worker, sequence: u64, payload: &str) -> String {
    worker.command(&format!("APPEND {sequence} {payload}"))
}

fn assert_status(worker: &mut Worker, count: usize, last: u64) {
    assert_eq!(worker.command("STATUS"), format!("STATUS {count} {last}"));
}

fn replicated_scenario(root: &Path) -> Vec<Vec<u8>> {
    let mut workers: Vec<Worker> = (0..4)
        .map(|id| Worker::start(id, &root.join(format!("node-{id}.log"))))
        .collect();

    // Initial common prefix.
    for worker in &mut workers {
        assert_eq!(append(worker, 1, "aa"), "APPENDED");
    }

    // Partition node 3. Deliver sequence 3 early to demonstrate reordering.
    assert_eq!(append(&mut workers[3], 3, "cc"), "GAP");

    // Nodes 0, 1 and 2 receive sequence 2. Node 2 is then deliberately
    // withheld from sequence 3 while the other two advance.
    for worker in &mut workers[..3] {
        assert_eq!(append(worker, 2, "bb"), "APPENDED");
    }
    assert_eq!(append(&mut workers[0], 3, "cc"), "APPENDED");
    assert_eq!(append(&mut workers[1], 3, "cc"), "APPENDED");

    // A conflicting value at an already accepted coordinate is rejected.
    assert_eq!(append(&mut workers[0], 3, "dd"), "CONFLICT");

    // Heal the partition and deliver the missing/delayed messages in order.
    assert_eq!(append(&mut workers[2], 3, "cc"), "APPENDED");
    assert_eq!(append(&mut workers[3], 2, "bb"), "APPENDED");
    assert_eq!(append(&mut workers[3], 3, "cc"), "APPENDED");

    for worker in &mut workers {
        assert_status(worker, 3, 3);
    }
    for worker in workers {
        worker.stop();
    }

    (0..4)
        .map(|id| fs::read(root.join(format!("node-{id}.log"))).unwrap())
        .collect()
}

#[test]
fn four_independent_processes_recover_from_partition_drop_reorder_and_conflict() {
    let directory = TestDir::new("partition");
    let logs = replicated_scenario(directory.path());
    assert!(logs.windows(2).all(|pair| pair[0] == pair[1]));
}

#[test]
fn process_restart_recovers_journal_and_replacement_node_replays_history() {
    let directory = TestDir::new("restart-churn");
    let mut workers: Vec<Worker> = (0..4)
        .map(|id| Worker::start(id, &directory.path().join(format!("node-{id}.log"))))
        .collect();

    for worker in &mut workers {
        assert_eq!(append(worker, 1, "10"), "APPENDED");
        assert_eq!(append(worker, 2, "20"), "APPENDED");
    }

    // Stop and restart one independent process against its existing journal.
    workers.swap_remove(2).stop();
    let mut restarted = Worker::start(2, &directory.path().join("node-2.log"));
    assert_status(&mut restarted, 2, 2);
    assert_eq!(append(&mut restarted, 3, "30"), "APPENDED");

    // Existing validators receive the same next record.
    for worker in &mut workers {
        assert_eq!(append(worker, 3, "30"), "APPENDED");
    }

    // Churn: a replacement process starts empty and catches up by replay.
    let mut replacement = Worker::start(4, &directory.path().join("node-4.log"));
    for (sequence, payload) in [(1, "10"), (2, "20"), (3, "30")] {
        assert_eq!(append(&mut replacement, sequence, payload), "APPENDED");
    }
    assert_status(&mut replacement, 3, 3);

    for worker in workers {
        worker.stop();
    }
    restarted.stop();
    replacement.stop();

    let expected = fs::read(directory.path().join("node-0.log")).unwrap();
    assert_eq!(fs::read(directory.path().join("node-1.log")).unwrap(), expected);
    assert_eq!(fs::read(directory.path().join("node-2.log")).unwrap(), expected);
    assert_eq!(fs::read(directory.path().join("node-4.log")).unwrap(), expected);
}

#[test]
fn deterministic_replay_and_per_message_resource_bounds() {
    let first = TestDir::new("replay-a");
    let second = TestDir::new("replay-b");
    let first_logs = replicated_scenario(first.path());
    let second_logs = replicated_scenario(second.path());
    assert_eq!(first_logs, second_logs);

    let limits = TestDir::new("limits");
    let mut worker = Worker::start(99, &limits.path().join("node.log"));
    assert_eq!(
        append(&mut worker, 1, &"ab".repeat(513)),
        "ERR_PAYLOAD_TOO_LARGE"
    );
    assert_eq!(worker.command("STATUS"), "STATUS 0 0");
    assert_eq!(append(&mut worker, 1, "ab"), "APPENDED");
    worker.stop();
}
