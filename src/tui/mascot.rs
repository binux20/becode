//! 🐝 BeCode Mascot - the friendly coding bee!

use rand::Rng;

/// Mascot states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MascotState {
    Idle,
    Thinking,
    Working,
    Success,
    Error,
    Waiting,
}

/// The BeCode bee mascot
pub struct Mascot {
    pub state: MascotState,
    pub enabled: bool,
    pub phrases_enabled: bool,
}

impl Default for Mascot {
    fn default() -> Self {
        Self {
            state: MascotState::Idle,
            enabled: true,
            phrases_enabled: true,
        }
    }
}

impl Mascot {
    /// Get emoji for current state
    pub fn emoji(&self) -> &'static str {
        match self.state {
            MascotState::Idle => "🐝",
            MascotState::Thinking => "🤔",
            MascotState::Working => "🔨",
            MascotState::Success => "✅",
            MascotState::Error => "😅",
            MascotState::Waiting => "⏳",
        }
    }

    /// Get a random phrase for current state
    pub fn phrase(&self) -> &'static str {
        if !self.phrases_enabled {
            return "";
        }

        let phrases = match self.state {
            MascotState::Idle => IDLE_PHRASES,
            MascotState::Thinking => THINKING_PHRASES,
            MascotState::Working => WORKING_PHRASES,
            MascotState::Success => SUCCESS_PHRASES,
            MascotState::Error => ERROR_PHRASES,
            MascotState::Waiting => WAITING_PHRASES,
        };

        let mut rng = rand::thread_rng();
        phrases[rng.gen_range(0..phrases.len())]
    }

    /// Set state
    pub fn set_state(&mut self, state: MascotState) {
        self.state = state;
    }
}

// Phrase collections
const IDLE_PHRASES: &[&str] = &[
    "Ready to help! 🐝",
    "What shall we build today?",
    "Bzz... awaiting your command!",
    "The hive is ready!",
    "Let's make some honey (code)!",
];

const THINKING_PHRASES: &[&str] = &[
    "Bzz... analyzing the codebase...",
    "Let me think about this...",
    "Processing with bee-level intelligence...",
    "Consulting the hive mind...",
    "Pollinating ideas...",
    "Searching through the honeycomb...",
    "Compiling thoughts...",
    "Running mental unit tests...",
];

const WORKING_PHRASES: &[&str] = &[
    "Building the honeycomb...",
    "Writing some sweet code...",
    "Buzzing through the files...",
    "Crafting with precision...",
    "Making changes...",
    "Applying bee-st practices...",
    "Refactoring the hive...",
    "Optimizing the nectar flow...",
];

const SUCCESS_PHRASES: &[&str] = &[
    "Done! 🍯",
    "Mission accomplished!",
    "The hive approves!",
    "Sweet success!",
    "Another flower pollinated!",
    "Bzz-eautiful work!",
    "The code is now bee-utiful!",
];

const ERROR_PHRASES: &[&str] = &[
    "Oops, hit a wall...",
    "The bee got lost...",
    "Something stung us...",
    "Need to try another flower...",
    "The honey got sticky...",
    "Let me try a different approach...",
];

const WAITING_PHRASES: &[&str] = &[
    "Waiting patiently...",
    "Standing by...",
    "Hovering in place...",
    "Ready when you are!",
    "Taking a quick rest...",
];

/// ASCII art bee for the `becode bee` command
pub const BEE_ASCII_ART: &str = r#"
              \     /
          \    o ^ o    /
            \ (     ) /
 ____________(%%%%%%%)____________
(     /   /  )%%%%%%%(  \   \     )
(___/___/__/           \__\___\___)
   (     /  /(%%%%%%%)\  \     )
    (__/__(////////////)__\__)
            `\ ║════║ /'
              `\    /'
                `\/'

         🐝 BeCode v2.0.0 🐝
    Your friendly coding companion!

  "Turning bugs into features since 2024"
"#;

/// Alternative smaller bee
pub const BEE_SMALL: &str = r#"
    \ _ /
  -= (_) =-
    /   \      🐝
"#;

/// Animated bee frames for loading
pub const BEE_ANIMATION: &[&str] = &[
    "  🐝    ",
    "   🐝   ",
    "    🐝  ",
    "     🐝 ",
    "    🐝  ",
    "   🐝   ",
    "  🐝    ",
    " 🐝     ",
];
