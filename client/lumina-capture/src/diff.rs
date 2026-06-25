use crate::Frame;

#[derive(Debug, Clone, PartialEq)]
pub struct Rect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug)]
pub struct DiffResult {
    pub has_changes: bool,
    pub dirty_rect: Option<Rect>,
}

/// Computes the bounding box of changed pixels between two frames.
/// Optimized by comparing 4-byte chunks (RGBA pixels) to trigger LLVM autovectorization.
pub fn compute_dirty_rect(old_frame: &Frame, new_frame: &Frame) -> DiffResult {
    let width = new_frame.width;
    let height = new_frame.height;

    // Safety fallback: if dimensions mismatch or lengths differ, assume full redraw
    if old_frame.width != width || old_frame.height != height || old_frame.data.len() != new_frame.data.len() {
        return DiffResult {
            has_changes: true,
            dirty_rect: Some(Rect { x: 0, y: 0, width, height }),
        };
    }

    let mut min_x = width;
    let mut min_y = height;
    let mut max_x = 0;
    let mut max_y = 0;
    let mut changed = false;

    // Process row by row for cache locality
    for y in 0..height {
        let mut row_changed = false;
        
        let row_start = (y * width * 4) as usize;
        let row_end = ((y + 1) * width * 4) as usize;
        
        let old_row = &old_frame.data[row_start..row_end];
        let new_row = &new_frame.data[row_start..row_end];
        
        // Fast comparison: 1 pixel = 4 bytes (RGBA)
        for (x, (old_px, new_px)) in old_row.chunks_exact(4).zip(new_row.chunks_exact(4)).enumerate() {
            if old_px != new_px {
                let x = x as u32;
                if x < min_x { min_x = x; }
                if x > max_x { max_x = x; }
                row_changed = true;
            }
        }

        if row_changed {
            if y < min_y { min_y = y; }
            if y > max_y { max_y = y; }
            changed = true;
        }
    }

    if changed {
        DiffResult {
            has_changes: true,
            dirty_rect: Some(Rect {
                x: min_x,
                y: min_y,
                width: max_x - min_x + 1,
                height: max_y - min_y + 1,
            }),
        }
    } else {
        DiffResult {
            has_changes: false,
            dirty_rect: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn create_blank_frame() -> Frame {
        Frame {
            width: 100,
            height: 100,
            data: vec![0; 100 * 100 * 4],
            timestamp: Duration::from_secs(0),
        }
    }

    #[test]
    fn test_no_changes() {
        let frame1 = create_blank_frame();
        let frame2 = create_blank_frame();
        let diff = compute_dirty_rect(&frame1, &frame2);
        assert!(!diff.has_changes);
        assert_eq!(diff.dirty_rect, None);
    }

    #[test]
    fn test_partial_changes() {
        let frame1 = create_blank_frame();
        let mut frame2 = create_blank_frame();
        
        // Change pixel at (10, 10)
        let idx1 = (10 * 100 + 10) * 4;
        frame2.data[idx1] = 255;
        
        // Change pixel at (20, 20)
        let idx2 = (20 * 100 + 20) * 4;
        frame2.data[idx2] = 255;

        let diff = compute_dirty_rect(&frame1, &frame2);
        assert!(diff.has_changes);
        
        let rect = diff.dirty_rect.unwrap();
        assert_eq!(rect.x, 10);
        assert_eq!(rect.y, 10);
        assert_eq!(rect.width, 11); // 20 - 10 + 1
        assert_eq!(rect.height, 11);
    }
}
