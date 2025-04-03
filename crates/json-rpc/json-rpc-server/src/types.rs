use std::cmp::Ordering;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ProcessPriority {
    High,
    Normal,
    Low,
    Custom(u8),
}

impl PartialOrd for ProcessPriority {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ProcessPriority {
    fn cmp(&self, other: &Self) -> Ordering {
        fn priority_value(p: &ProcessPriority) -> u8 {
            match p {
                ProcessPriority::High => 5,
                ProcessPriority::Normal => 3,
                ProcessPriority::Low => 1,
                ProcessPriority::Custom(v) => *v,
            }
        }

        priority_value(self).cmp(&priority_value(other))
    }
}

impl std::fmt::Display for ProcessPriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProcessPriority::High => write!(f, "High"),
            ProcessPriority::Normal => write!(f, "Normal"),
            ProcessPriority::Low => write!(f, "Low"),
            ProcessPriority::Custom(v) => write!(f, "Custom({})", v),
        }
    }
}

impl Default for ProcessPriority {
    fn default() -> Self {
        ProcessPriority::Normal
    }
}

pub struct PrioritizedRpcTask {
    pub priority: ProcessPriority,
    pub sequence: u64,
    pub job: Box<dyn FnOnce() + Send>,
}

impl Ord for PrioritizedRpcTask {
    fn cmp(&self, other: &Self) -> Ordering {
        self.priority
            .cmp(&other.priority)
            .then_with(|| other.sequence.cmp(&self.sequence)) // FIFO
    }
}

impl PartialEq for PrioritizedRpcTask {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority && self.sequence == other.sequence
    }
}

impl Eq for PrioritizedRpcTask {}

impl PartialOrd for PrioritizedRpcTask {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
