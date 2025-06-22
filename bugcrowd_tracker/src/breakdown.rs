static CRITICAL: i32 = 40;
static HIGH: i32 = 20;
static MODERATE: i32 = 10;
static LOW: i32 = 5;

#[derive(Default, Debug, PartialEq, Eq)]
pub struct PointsBreakdown {
    critical_bounty: i32,
    high_bounty: i32,
    moderate_bounty: i32,
    low_bounty: i32,
}

impl ToString for PointsBreakdown {
    fn to_string(&self) -> String {
        let mut parts = Vec::new();

        macro_rules! add_part {
            ($condition:expr, $text:expr, $count:expr) => {
                if $condition > 0 {
                    parts.push(if $count < 2 {
                        $text.to_string()
                    } else {
                        format!("{}({})", $text, $count)
                    });
                }
            };
        }

        add_part!(self.critical_bounty, "Critical", self.critical_bounty);
        add_part!(self.high_bounty, "High", self.high_bounty);
        add_part!(self.moderate_bounty, "Medium", self.moderate_bounty);
        add_part!(self.low_bounty, "Low", self.low_bounty);

        parts.join(", ")
    }
}

pub fn calculate_points_breakdown(mut points: i32) -> Option<PointsBreakdown> {
    let mut breakdown = PointsBreakdown::default();

    let thresholds = [CRITICAL, HIGH, MODERATE, LOW];
    let mut index = 0;

    while index < thresholds.len() {
        let is_over = {
            if thresholds[index] > 0 {
                points >= thresholds[index]
            } else {
                points <= thresholds[index]
            }
        };

        if is_over {
            let count = points / thresholds[index];
            match index {
                0 => breakdown.critical_bounty = count,
                1 => breakdown.high_bounty = count,
                2 => breakdown.moderate_bounty = count,
                3 => breakdown.low_bounty = count,
                _ => {}
            }

            points -= count * thresholds[index];
        }

        index += 1;
    }

    if points != 0 || breakdown == PointsBreakdown::default() {
        return None;
    }

    Some(breakdown)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple() {
        let a = calculate_points_breakdown(40);
        let b = calculate_points_breakdown(40 + 40);

        assert!(a.unwrap().to_string() == "Critical");
        assert!(b.unwrap().to_string() == "Critical(2)");
    }

    #[test]
    fn none() {
        let a = calculate_points_breakdown(0);
        let b = calculate_points_breakdown(3);

        assert!(a.is_none());
        assert!(b.is_none());
    }
}
