use serde::{Deserialize, Serialize};

/// PickStop represents a single stop in the picking route.
/// Matches Go's `PickStop` from `internal/services/picking/route.go`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PickStop {
    pub line_ids: Vec<i64>,
    pub rack_id: i64,
    pub rack_x: i32,
    pub rack_y: i32,
    pub center_x: f64,
    pub center_y: f64,
}

/// PathPoint is a coordinate on the route path.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathPoint {
    pub x: i32,
    pub y: i32,
}

/// RouteResult contains the optimized route.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteResult {
    pub stops: Vec<PickStop>,
    pub total_distance_px: f64,
    pub path: Vec<PathPoint>,
}

/// Orders pick stops using nearest-neighbor TSP heuristic.
/// `start_x`, `start_y` is the warehouse entrance (typically 0,0).
/// Returns ordered stops and total walking distance in pixels.
pub fn calculate_route(stops: &[PickStop], start_x: i32, start_y: i32) -> RouteResult {
    if stops.is_empty() {
        return RouteResult {
            stops: vec![],
            total_distance_px: 0.0,
            path: vec![],
        };
    }

    if stops.len() == 1 {
        let dist = euclidean(
            start_x as f64,
            start_y as f64,
            stops[0].center_x,
            stops[0].center_y,
        );
        return RouteResult {
            stops: stops.to_vec(),
            total_distance_px: dist,
            path: vec![
                PathPoint { x: start_x, y: start_y },
                PathPoint {
                    x: stops[0].center_x as i32,
                    y: stops[0].center_y as i32,
                },
            ],
        };
    }

    let mut remaining: Vec<PickStop> = stops.to_vec();
    let mut ordered = Vec::with_capacity(stops.len());
    let mut path = vec![PathPoint { x: start_x, y: start_y }];
    let mut total_dist = 0.0;

    let mut cur_x = start_x as f64;
    let mut cur_y = start_y as f64;

    while !remaining.is_empty() {
        let mut nearest_idx = 0;
        let mut nearest_dist = f64::MAX;

        for (i, stop) in remaining.iter().enumerate() {
            let dist = euclidean(cur_x, cur_y, stop.center_x, stop.center_y);
            if dist < nearest_dist {
                nearest_dist = dist;
                nearest_idx = i;
            }
        }

        // Swap-remove matches Go's removal pattern (swap last element in)
        let nearest = remaining.swap_remove(nearest_idx);

        cur_x = nearest.center_x;
        cur_y = nearest.center_y;

        path.push(PathPoint {
            x: cur_x as i32,
            y: cur_y as i32,
        });

        total_dist += nearest_dist;
        ordered.push(nearest);
    }

    RouteResult {
        stops: ordered,
        total_distance_px: total_dist,
        path,
    }
}

fn euclidean(x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
    let dx = x2 - x1;
    let dy = y2 - y1;
    (dx * dx + dy * dy).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_stops() {
        let result = calculate_route(&[], 0, 0);
        assert!(result.stops.is_empty());
        assert_eq!(result.total_distance_px, 0.0);
        assert!(result.path.is_empty());
    }

    #[test]
    fn test_single_stop() {
        let stops = vec![PickStop {
            line_ids: vec![1],
            rack_id: 10,
            rack_x: 100,
            rack_y: 200,
            center_x: 100.0,
            center_y: 0.0,
        }];
        let result = calculate_route(&stops, 0, 0);
        assert_eq!(result.stops.len(), 1);
        assert_eq!(result.total_distance_px, 100.0);
        assert_eq!(result.path.len(), 2);
    }

    #[test]
    fn test_nearest_neighbor_order() {
        // Three stops: far(300,0), mid(200,0), near(100,0) from origin
        let stops = vec![
            PickStop {
                line_ids: vec![3],
                rack_id: 30,
                rack_x: 300,
                rack_y: 0,
                center_x: 300.0,
                center_y: 0.0,
            },
            PickStop {
                line_ids: vec![1],
                rack_id: 10,
                rack_x: 100,
                rack_y: 0,
                center_x: 100.0,
                center_y: 0.0,
            },
            PickStop {
                line_ids: vec![2],
                rack_id: 20,
                rack_x: 200,
                rack_y: 0,
                center_x: 200.0,
                center_y: 0.0,
            },
        ];
        let result = calculate_route(&stops, 0, 0);
        // Should visit in order: near(100) -> mid(200) -> far(300)
        assert_eq!(result.stops[0].rack_id, 10);
        assert_eq!(result.stops[1].rack_id, 20);
        assert_eq!(result.stops[2].rack_id, 30);
        assert_eq!(result.total_distance_px, 300.0);
    }

    #[test]
    fn test_diagonal_distance() {
        let stops = vec![PickStop {
            line_ids: vec![1],
            rack_id: 1,
            rack_x: 3,
            rack_y: 4,
            center_x: 3.0,
            center_y: 4.0,
        }];
        let result = calculate_route(&stops, 0, 0);
        assert!((result.total_distance_px - 5.0).abs() < 1e-10); // 3-4-5 triangle
    }
}
