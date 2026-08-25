use crate::{MAX_RUN_SPEED, SLIDE_DURATION};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub enum Difficulty {
    Easy,
    #[default]
    Normal,
    Hard,
}

impl Difficulty {
    pub fn label(self) -> &'static str {
        match self {
            Self::Easy => "EASY",
            Self::Normal => "NORMAL",
            Self::Hard => "HARD",
        }
    }

    pub fn start_speed(self) -> f32 {
        match self {
            Self::Easy => 250.0,
            Self::Normal => 310.0,
            Self::Hard => 380.0,
        }
    }

    pub fn speed_gain(self) -> f32 {
        match self {
            Self::Easy => 0.018,
            Self::Normal => 0.028,
            Self::Hard => 0.038,
        }
    }

    pub fn stumble_threat(self) -> f32 {
        match self {
            Self::Easy => 0.28,
            Self::Normal => 0.40,
            Self::Hard => 0.55,
        }
    }

    pub fn recover_rate(self) -> f32 {
        match self {
            Self::Easy => 0.22,
            Self::Normal => 0.16,
            Self::Hard => 0.11,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DeathReason {
    #[default]
    Fell,
    Caught,
    Crash,
}

impl DeathReason {
    pub fn headline(self) -> &'static str {
        match self {
            Self::Fell => "YOU FELL INTO THE CITY",
            Self::Caught => "THE HORDE CAUGHT YOU",
            Self::Crash => "WIPED OUT",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObstacleKind {
    LowBarrier,
    HighBarrier,
    LaneBlock,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerUpKind {
    Magnet,
    Shield,
    Multiplier,
    Boost,
}

impl PowerUpKind {
    pub fn duration(self) -> f32 {
        match self {
            Self::Magnet | Self::Multiplier => 8.0,
            Self::Boost => 3.0,
            Self::Shield => 0.0,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Magnet => "COIN MAGNET",
            Self::Shield => "SHIELD",
            Self::Multiplier => "x2 COINS",
            Self::Boost => "NITRO BOOST",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HitOutcome {
    Shielded,
    Stumble,
    Death,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SegmentPlan {
    Safe { length: f32 },
    Coins { lane: i32, count: u32 },
    LowBarriers { lanes: Vec<i32> },
    HighBars { lanes: Vec<i32> },
    Gap { length: f32 },
    PowerUp { kind: PowerUpKind, lane: i32 },
    JumpSet { obstacle_lane: i32, coin_lane: i32 },
    SlideSet { obstacle_lane: i32, coin_lane: i32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CoinAward {
    pub coins: u32,
    pub points: u32,
}

pub fn clamp_lane(lane: i32) -> i32 {
    lane.clamp(0, 2)
}

pub fn next_lane(current: i32, delta: i32) -> i32 {
    clamp_lane(current + delta)
}

pub fn lane_from_blend(lane_blend: f32) -> i32 {
    clamp_lane(lane_blend.round() as i32)
}

pub fn run_speed(distance: f32, difficulty: Difficulty, boost: bool) -> f32 {
    let base = (difficulty.start_speed() + distance * difficulty.speed_gain()).min(MAX_RUN_SPEED);
    if boost {
        (base * 1.45).min(MAX_RUN_SPEED + 80.0)
    } else {
        base
    }
}

pub fn threat_after_stumble(threat: f32, bump: f32) -> f32 {
    (threat + bump).clamp(0.0, 1.0)
}

pub fn threat_recover(threat: f32, dt: f32, rate: f32) -> f32 {
    (threat - rate * dt).clamp(0.0, 1.0)
}

pub fn is_caught(threat: f32) -> bool {
    threat >= 1.0
}

pub fn displayed_score(distance: f32, coin_points: u32) -> u64 {
    (distance / 10.0).max(0.0) as u64 + coin_points as u64
}

pub fn award_coins(base_value: u32, multiplier_active: bool) -> CoinAward {
    let coins = base_value.max(1);
    let points = if multiplier_active {
        coins * 20
    } else {
        coins * 10
    };
    CoinAward { coins, points }
}

pub fn obstacle_cleared(kind: ObstacleKind, jumping: bool, sliding: bool) -> bool {
    match kind {
        ObstacleKind::LowBarrier => jumping,
        ObstacleKind::HighBarrier => sliding,
        ObstacleKind::LaneBlock => false,
    }
}

pub fn resolve_hit(already_stumbling: bool, has_shield: bool) -> HitOutcome {
    if has_shield {
        HitOutcome::Shielded
    } else if already_stumbling {
        HitOutcome::Death
    } else {
        HitOutcome::Stumble
    }
}

pub fn parse_highscore(raw: &str) -> Option<(u64, u32, f32)> {
    let mut parts = raw.split_whitespace();
    let score = parts.next()?.parse().ok()?;
    let coins = parts.next()?.parse().ok()?;
    let distance = parts.next()?.parse().ok()?;
    if score == 0 && coins == 0 && distance == 0.0 {
        return Some((0, 0, 0.0));
    }
    if distance < 0.0 {
        return None;
    }
    Some((score, coins, distance))
}

pub fn format_highscore(score: u64, coins: u32, distance: f32) -> String {
    format!("{score} {coins} {distance:.1}\n")
}

pub fn is_new_highscore(previous: u64, current: u64) -> bool {
    current > previous
}

pub fn slide_time_left(elapsed: f32) -> f32 {
    (SLIDE_DURATION - elapsed).max(0.0)
}

/// Deterministic segment picker. `roll` is 0..1, `lane_roll` is 0..1, `kind_roll` is 0..1.
pub fn plan_segment(
    distance: f32,
    difficulty: Difficulty,
    roll: f32,
    lane_roll: f32,
    kind_roll: f32,
) -> SegmentPlan {
    let roll = roll.clamp(0.0, 0.999);
    let lane = clamp_lane((lane_roll.clamp(0.0, 0.999) * 3.0).floor() as i32);
    let other_lane = clamp_lane(lane + 1);
    let density = match difficulty {
        Difficulty::Easy => 0.0,
        Difficulty::Normal => 0.08,
        Difficulty::Hard => 0.16,
    };
    let intro = (1.0 - (distance / 800.0).clamp(0.0, 1.0)) * 0.22;

    if roll < 0.10 + intro {
        SegmentPlan::Safe {
            length: 280.0 + kind_roll * 180.0,
        }
    } else if roll < 0.28 + intro {
        SegmentPlan::Coins {
            lane,
            count: 4 + ((kind_roll * 5.0) as u32),
        }
    } else if roll < 0.40 + density {
        SegmentPlan::JumpSet {
            obstacle_lane: lane,
            coin_lane: other_lane,
        }
    } else if roll < 0.52 + density {
        SegmentPlan::SlideSet {
            obstacle_lane: lane,
            coin_lane: other_lane,
        }
    } else if roll < 0.66 + density {
        let second = if kind_roll > 0.55 && difficulty != Difficulty::Easy {
            vec![lane, other_lane]
        } else {
            vec![lane]
        };
        SegmentPlan::LowBarriers { lanes: second }
    } else if roll < 0.80 + density {
        let second = if kind_roll > 0.60 && difficulty != Difficulty::Easy {
            vec![lane, other_lane]
        } else {
            vec![lane]
        };
        SegmentPlan::HighBars { lanes: second }
    } else if roll < 0.90 + density * 0.5 && distance > 400.0 {
        SegmentPlan::Gap {
            length: 90.0 + kind_roll * 70.0,
        }
    } else if roll < 0.96 {
        let kind = match (kind_roll * 4.0).floor() as i32 {
            0 => PowerUpKind::Magnet,
            1 => PowerUpKind::Shield,
            2 => PowerUpKind::Multiplier,
            _ => PowerUpKind::Boost,
        };
        SegmentPlan::PowerUp { kind, lane }
    } else {
        SegmentPlan::Safe {
            length: 220.0 + kind_roll * 80.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lane_stays_inside_three_tracks() {
        assert_eq!(clamp_lane(-4), 0);
        assert_eq!(clamp_lane(7), 2);
        assert_eq!(next_lane(0, -1), 0);
        assert_eq!(next_lane(0, 1), 1);
        assert_eq!(next_lane(2, 1), 2);
        assert_eq!(lane_from_blend(0.4), 0);
        assert_eq!(lane_from_blend(1.6), 2);
    }

    #[test]
    fn speed_ramps_with_distance_and_difficulty() {
        let easy = run_speed(0.0, Difficulty::Easy, false);
        let hard = run_speed(0.0, Difficulty::Hard, false);
        assert!(hard > easy);
        assert!(
            run_speed(4000.0, Difficulty::Normal, false)
                > run_speed(0.0, Difficulty::Normal, false)
        );
        assert!(
            run_speed(500.0, Difficulty::Normal, true)
                > run_speed(500.0, Difficulty::Normal, false)
        );
        assert!(run_speed(100_000.0, Difficulty::Hard, false) <= MAX_RUN_SPEED);
    }

    #[test]
    fn threat_and_catch_rules() {
        assert_eq!(threat_after_stumble(0.2, 0.4), 0.6);
        assert_eq!(threat_after_stumble(0.9, 0.4), 1.0);
        assert!((threat_recover(0.5, 1.0, 0.2) - 0.3).abs() < f32::EPSILON);
        assert!(!is_caught(0.99));
        assert!(is_caught(1.0));
    }

    #[test]
    fn score_and_coin_awards() {
        assert_eq!(displayed_score(250.0, 40), 65);
        assert_eq!(
            award_coins(1, false),
            CoinAward {
                coins: 1,
                points: 10
            }
        );
        assert_eq!(
            award_coins(1, true),
            CoinAward {
                coins: 1,
                points: 20
            }
        );
    }

    #[test]
    fn obstacle_clearance_matches_temple_run_verbs() {
        assert!(obstacle_cleared(ObstacleKind::LowBarrier, true, false));
        assert!(!obstacle_cleared(ObstacleKind::LowBarrier, false, true));
        assert!(obstacle_cleared(ObstacleKind::HighBarrier, false, true));
        assert!(!obstacle_cleared(ObstacleKind::HighBarrier, true, false));
        assert!(!obstacle_cleared(ObstacleKind::LaneBlock, true, true));
    }

    #[test]
    fn hit_resolution_uses_shield_then_stumble_then_death() {
        assert_eq!(resolve_hit(false, true), HitOutcome::Shielded);
        assert_eq!(resolve_hit(true, true), HitOutcome::Shielded);
        assert_eq!(resolve_hit(false, false), HitOutcome::Stumble);
        assert_eq!(resolve_hit(true, false), HitOutcome::Death);
    }

    #[test]
    fn highscore_roundtrip_and_compare() {
        let raw = format_highscore(1234, 56, 890.4);
        assert_eq!(parse_highscore(&raw), Some((1234, 56, 890.4)));
        assert_eq!(parse_highscore("bad data"), None);
        assert_eq!(parse_highscore("-1 0 1"), None);
        assert!(is_new_highscore(10, 11));
        assert!(!is_new_highscore(10, 10));
    }

    #[test]
    fn early_runs_prefer_safe_or_coin_segments() {
        let plan = plan_segment(0.0, Difficulty::Easy, 0.05, 0.1, 0.2);
        assert!(matches!(plan, SegmentPlan::Safe { .. }));
        let coins = plan_segment(50.0, Difficulty::Easy, 0.35, 0.4, 0.5);
        assert!(matches!(coins, SegmentPlan::Coins { lane: 1, .. }));
    }

    #[test]
    fn later_hard_runs_can_open_gaps() {
        let gap = plan_segment(1200.0, Difficulty::Hard, 0.97, 0.0, 0.3);
        assert!(matches!(gap, SegmentPlan::Gap { .. }));
    }

    #[test]
    fn slide_timer_never_goes_negative() {
        assert_eq!(slide_time_left(0.0), SLIDE_DURATION);
        assert_eq!(slide_time_left(10.0), 0.0);
    }

    #[test]
    fn death_copy_is_temple_run_styled() {
        assert!(DeathReason::Caught.headline().contains("HORDE"));
        assert!(DeathReason::Fell.headline().contains("FELL"));
    }
}
