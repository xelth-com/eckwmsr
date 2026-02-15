use crate::models::sync_packet::EntityMetadata;
use crate::sync::vector_clock::ClockRelation;

#[derive(Debug, PartialEq)]
pub enum Resolution {
    /// Remote is newer/better — apply it
    ApplyRemote,
    /// Local is newer/better — keep it
    KeepLocal,
    /// Concurrent edit — needs rule-based resolution
    Conflict,
}

pub struct ConflictResolver {
    #[allow(dead_code)]
    instance_id: String,
}

impl ConflictResolver {
    pub fn new(instance_id: String) -> Self {
        Self { instance_id }
    }

    /// Determines if a remote change should be accepted over local data.
    /// Uses a 3-tier resolution strategy:
    ///   1. Vector Clock causality (definitive if not concurrent)
    ///   2. Source priority (Odoo > PDA > manual)
    ///   3. Last-Write-Wins timestamp (fallback)
    pub fn resolve(&self, local: &EntityMetadata, remote: &EntityMetadata) -> Resolution {
        // 1. Vector Clock check (causality)
        match local.vector_clock.compare(&remote.vector_clock) {
            ClockRelation::Before => return Resolution::ApplyRemote,
            ClockRelation::After => return Resolution::KeepLocal,
            ClockRelation::Equal => {
                // Identical clocks — prefer newer timestamp
                if remote.updated_at > local.updated_at {
                    return Resolution::ApplyRemote;
                }
                return Resolution::KeepLocal;
            }
            ClockRelation::Concurrent => {
                // Fall through to priority-based resolution
            }
        }

        // 2. Source priority (higher = more authoritative)
        if remote.source_priority > local.source_priority {
            return Resolution::ApplyRemote;
        }
        if local.source_priority > remote.source_priority {
            return Resolution::KeepLocal;
        }

        // 3. Last-Write-Wins (LWW) timestamp fallback
        if remote.updated_at > local.updated_at {
            return Resolution::ApplyRemote;
        }

        Resolution::KeepLocal
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sync::vector_clock::VectorClock;
    use chrono::Utc;

    fn make_meta(vc: VectorClock, priority: i32, secs_offset: i64) -> EntityMetadata {
        EntityMetadata {
            entity_id: "test-1".into(),
            entity_type: "product".into(),
            version: 1,
            updated_at: Utc::now() + chrono::Duration::seconds(secs_offset),
            source: "test".into(),
            source_priority: priority,
            instance_id: "node-a".into(),
            device_id: None,
            vector_clock: vc,
        }
    }

    #[test]
    fn test_clock_before_applies_remote() {
        let mut vc_local = VectorClock::new();
        vc_local.increment("a");
        let mut vc_remote = vc_local.clone();
        vc_remote.increment("a");

        let local = make_meta(vc_local, 50, 0);
        let remote = make_meta(vc_remote, 50, 0);

        let resolver = ConflictResolver::new("a".into());
        assert_eq!(resolver.resolve(&local, &remote), Resolution::ApplyRemote);
    }

    #[test]
    fn test_clock_after_keeps_local() {
        let mut vc_remote = VectorClock::new();
        vc_remote.increment("b");
        let mut vc_local = vc_remote.clone();
        vc_local.increment("b");

        let local = make_meta(vc_local, 50, 0);
        let remote = make_meta(vc_remote, 50, 0);

        let resolver = ConflictResolver::new("a".into());
        assert_eq!(resolver.resolve(&local, &remote), Resolution::KeepLocal);
    }

    #[test]
    fn test_concurrent_higher_priority_wins() {
        let mut vc_a = VectorClock::new();
        vc_a.increment("a");
        let mut vc_b = VectorClock::new();
        vc_b.increment("b");

        let local = make_meta(vc_a, 50, 0);   // lower priority
        let remote = make_meta(vc_b, 80, 0);   // higher priority (e.g. Odoo)

        let resolver = ConflictResolver::new("a".into());
        assert_eq!(resolver.resolve(&local, &remote), Resolution::ApplyRemote);
    }

    #[test]
    fn test_concurrent_same_priority_lww() {
        let mut vc_a = VectorClock::new();
        vc_a.increment("a");
        let mut vc_b = VectorClock::new();
        vc_b.increment("b");

        let local = make_meta(vc_a, 50, 0);
        let remote = make_meta(vc_b, 50, 10); // 10 seconds newer

        let resolver = ConflictResolver::new("a".into());
        assert_eq!(resolver.resolve(&local, &remote), Resolution::ApplyRemote);
    }
}
