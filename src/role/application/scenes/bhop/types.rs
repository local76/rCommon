#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CommandState {
    Typing,
    Executing,
    CoolDown,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BhopState {
    Playing,
    Dead,
    Respawning,
}

pub struct LcgRng(u64);
impl LcgRng {
    pub fn new(seed: u64) -> Self {
        Self(seed | 1)
    }
    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        self.0
    }
    pub fn next_f32(&mut self) -> f32 {
        let val = self.next_u64() >> 11;
        (val as f64 / 9007199254740992.0) as f32
    }
    pub fn next_bool(&mut self, prob: f32) -> bool {
        self.next_f32() < prob
    }
}

pub const COMMANDS: &[(&str, &[&str])] = &[
    (
        "ping -c 3 trance-labs.com",
        &[
            "PING trance-labs.com (104.21.32.222): 56 data bytes",
            "64 bytes from 104.21.32.222: icmp_seq=0 ttl=56 time=12.4 ms",
            "64 bytes from 104.21.32.222: icmp_seq=1 ttl=56 time=14.1 ms",
            "64 bytes from 104.21.32.222: icmp_seq=2 ttl=56 time=11.8 ms",
            "--- trance-labs.com ping statistics ---",
            "3 packets transmitted, 3 received, 0% packet loss",
            "rtt min/avg/max = 11.8/12.7/14.1 ms",
        ],
    ),
    (
        "cargo build --release",
        &[
            "   Compiling once_cell v1.19.0",
            "   Compiling windows-sys v0.59.0",
            "   Compiling trance_screensaver v0.1.0",
            "    Finished `release` profile [optimized] target(s) in 3.42s",
        ],
    ),
    (
        "netstat -an | findstr :8080",
        &[
            "  TCP    0.0.0.0:8080           0.0.0.0:0              LISTENING",
            "  TCP    127.0.0.1:8080         127.0.0.1:54932        ESTABLISHED",
            "  TCP    127.0.0.1:54932        127.0.0.1:8080         ESTABLISHED",
        ],
    ),
    (
        "git status",
        &[
            "On branch main",
            "Your branch is up to date with 'origin/main'.",
            "Changes not staged for commit:",
            "  (use \"git add <file>...\" to update what will be committed)",
            "	modified:   src/animation.rs",
            "	modified:   src/renderer.rs",
            "no changes added to commit (use \"git add\" and/or \"git commit -a\")",
        ],
    ),
    (
        "cat /etc/passwd | grep trance",
        &[
            "trance:x:1001:1001:Windows Admin,,,:/home/trance:/bin/bash",
        ],
    ),
];
