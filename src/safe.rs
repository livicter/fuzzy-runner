use bevy::prelude::*;
use std::time::Duration;

/// Bevy's `Duration::from_secs_f32` panics on NaN / negative / non-finite values.
pub fn safe_duration(seconds: f32) -> Duration {
    if !seconds.is_finite() {
        return Duration::ZERO;
    }
    Duration::from_secs_f32(seconds.clamp(0.0, 86_400.0))
}

/// Build a timer without going through `Timer::from_seconds`, which panics on junk floats.
pub fn safe_timer(seconds: f32, mode: TimerMode) -> Timer {
    Timer::new(safe_duration(seconds), mode)
}

/// Despawn only if the entity is still alive. Bevy 0.13 panics if you command a missing id.
pub fn try_despawn(commands: &mut Commands, entity: Entity) {
    if let Some(entity) = commands.get_entity(entity) {
        entity.despawn_recursive();
    }
}

/// Write into the first text section when it exists. Empty `Text` is a no-op, not a panic.
pub fn set_text(text: &mut Text, value: impl Into<String>) {
    if let Some(section) = text.sections.first_mut() {
        section.value = value.into();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safe_duration_accepts_normal_seconds() {
        assert_eq!(safe_duration(0.25), Duration::from_secs_f32(0.25));
        assert_eq!(safe_duration(0.0), Duration::ZERO);
    }

    #[test]
    fn safe_duration_swallows_non_finite_and_negative() {
        assert_eq!(safe_duration(f32::NAN), Duration::ZERO);
        assert_eq!(safe_duration(f32::INFINITY), Duration::ZERO);
        assert_eq!(safe_duration(f32::NEG_INFINITY), Duration::ZERO);
        assert_eq!(safe_duration(-3.5), Duration::ZERO);
    }
}
