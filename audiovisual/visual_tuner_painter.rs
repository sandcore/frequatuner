use pitch_detector::{core::NoteName, note::NoteDetectionResult};
use super::graphics::*;
use crate::{LEDS_MAX_X, LEDS_MAX_Y};

/*
Based on a NoteDetectionResult from pitch_detector crate. One future refactor might be to keep an enum of interface elements so their types can be passed around easily and checked, 
and interface elements are structs and keep their offsets and colors stored in there. Would be better when interface gets more complex, overkill for current goals.
      
pub struct NoteDetectionResult 
    /// The predominant frequency detected from a signal.
    pub actual_freq: f64,

    /// The note name of the detected note.
    pub note_name: NoteName,

    /// The expected frequency of the detected note.
    pub note_freq: f64,

    /// The octave of the detected note.
    pub octave: i32,

    /// The degree to which the detected not is in tune, expressed in cents. The absolute maximum `cents_offset` is
    /// 50, since anything larger than 50 would be considered the next or previous note.
    pub cents_offset: f64,

    /// The note name of the note that comes before the detected note. Not commonly used.
    pub previous_note_name: NoteName,

    /// The note name of the note that comes after the detected note. Not commonly used.
    pub next_note_name: NoteName,

    /// A `NoteDetectionResult` will be marked as `in_tune` if the `cents_offset` is less than
    /// [`MAX_CENTS_OFFSET`](crate::core::constants::MAX_CENTS_OFFSET).
    pub in_tune: bool,
*/

/*
Painter keeps some general state and runs the steps to draw layers.
*/
pub struct Painter {}
impl Painter {
    pub fn paint(&mut self, note_det_result: &NoteDetectionResult) -> Vec<u8> {
        let detected_note = &note_det_result.note_name;
        let prev_note = &note_det_result.previous_note_name;
        let next_note = &note_det_result.next_note_name;
        let cents_offset = note_det_result.cents_offset;
        let in_tune = note_det_result.in_tune;

        println!("{} {} {} {} {}", &note_det_result.note_name, &note_det_result.previous_note_name, &note_det_result.next_note_name, &note_det_result.cents_offset, &note_det_result.in_tune);

        let blank_canvas = BlankCanvas::new();
        let base_lined = blank_canvas.draw_baseline();
        let detected_line_drawn = base_lined.draw_detected_line(cents_offset);
        let notes_drawn = detected_line_drawn.draw_notes(detected_note, prev_note, next_note, in_tune);

        notes_drawn.output()
    }
}

/*
Typestates:
 - Blank canvas
 - Draw baseline for note (the tune goal frequency)
 - Draw detected line for note (the estimate of the actual frequency being played)
 - Draw note names
*/
struct BlankCanvas {
    color_vec: Vec<RGB>,

    // setting for the line to draw
    base_line_color: RGB
}
struct BaseLined {
    color_vec: Vec<RGB>,

    // settings for the line to draw
    detected_line_color: RGB,
    baseline_row: usize,
}

struct DetectedLineDrawn {
    color_vec: Vec<RGB>,

    // settings for the note to draw
    detected_note_color: RGB,
    in_tune_color: RGB,
    adjacent_note_color: RGB,
    start_row_col_detected: (usize, usize),
    start_row_col_prev: (usize, usize),
    start_row_col_next: (usize, usize)
}

struct NotesDrawn {
    color_vec: Vec<RGB>,
}

impl BlankCanvas {
    pub fn new() -> BlankCanvas {
        let mut empty_canvas = Vec::with_capacity(LEDS_MAX_X*LEDS_MAX_Y);
        for _ in 0..(LEDS_MAX_X*LEDS_MAX_Y) {
            empty_canvas.push(RGB{r:1,g:1,b:5});
        }

        BlankCanvas {
            color_vec: empty_canvas,
            base_line_color: RGB{r:255,g:215,b:0}
        }
    }

    fn draw_baseline(mut self) -> BaseLined {
        let baseline_row = (LEDS_MAX_Y as f32 / 2.0).round() as usize; // draw line starting at light 1 in row 17 (index 16), fixed around the center of the vertically placed ledstrip
        let line_graphic = super::graphics::line(LEDS_MAX_X, RGB{r: self.base_line_color.r, g:self.base_line_color.g, b: self.base_line_color.b});
        paint_element_rgb(&mut self.color_vec, &line_graphic, 0i32, baseline_row as i32);

        BaseLined {
            detected_line_color: RGB{r:51,g:255,b:255},
            baseline_row,
            color_vec: self.color_vec
        }
    }
}

impl BaseLined {
    fn draw_detected_line(mut self, cents_offset: f64) -> DetectedLineDrawn {      
        // -1 because even number leds with baseline in middle -> max distance is 1 less at one side of the baseline 
        let max_distance = LEDS_MAX_Y - self.baseline_row - 1;

        // draw the line in the positive or negative direction at cents_offset divided by 50
        // because as soon as the offset is more than 50% a new note becomes the baseline
        let offset_distance = (max_distance as f64 * cents_offset / 50.0).round() as i16;
        let draw_row = (self.baseline_row as i16 + offset_distance) as usize;

        let line_graphic = super::graphics::line(LEDS_MAX_X, RGB{r: self.detected_line_color.r, g:self.detected_line_color.g, b: self.detected_line_color.b});
        paint_element_rgb(&mut self.color_vec, &line_graphic, 0i32, draw_row as i32);

        DetectedLineDrawn {
            color_vec: self.color_vec,
            detected_note_color: RGB{r:200, g:0, b: 0},
            in_tune_color: RGB{r:0, g:255, b:0},
            adjacent_note_color: RGB{r:100, g:0, b: 100},
            start_row_col_detected: (14, 1),
            start_row_col_prev: (1, 1),
            start_row_col_next: (25, 1)
        }
    }
}

impl DetectedLineDrawn {
    fn draw_notes(mut self, detected_note: &NoteName, prev_note: &NoteName, next_note: &NoteName, in_tune: bool) -> NotesDrawn {
        let graphical_detected_note = GraphicalNote::new(detected_note);
        let graphical_prev_note = GraphicalNote::new(prev_note);
        let graphical_next_note = GraphicalNote::new(next_note);
        
        let detected_note_color = if in_tune {self.in_tune_color} else {self.detected_note_color};
        let adjacent_note_color = RGB{r:self.adjacent_note_color.r, g:self.adjacent_note_color.g, b:self.adjacent_note_color.b};
        
        let detected_graphic = convert_vecvecbool_to_xy_rgb_vec(graphical_detected_note.matrix, detected_note_color);
        let prev_note_graphic = convert_vecvecbool_to_xy_rgb_vec(graphical_prev_note.matrix, RGB{r:adjacent_note_color.r, g:adjacent_note_color.g, b:adjacent_note_color.b});
        let next_note_graphic = convert_vecvecbool_to_xy_rgb_vec(graphical_next_note.matrix, adjacent_note_color);
        
        paint_element_rgb(&mut self.color_vec, &detected_graphic, self.start_row_col_detected.1 as i32, self.start_row_col_detected.0 as i32);
        paint_element_rgb(&mut self.color_vec, &prev_note_graphic, self.start_row_col_prev.1 as i32, self.start_row_col_prev.0 as i32);
        paint_element_rgb(&mut self.color_vec, &next_note_graphic, self.start_row_col_next.1 as i32, self.start_row_col_next.0 as i32);

        NotesDrawn {
            color_vec: self.color_vec
        }
    }
}

impl NotesDrawn {
    fn output(self) -> Vec<u8> {
        // led matrix needs a vec of separate GRB values
        self.color_vec.into_iter().flat_map(|rgb| [rgb.g, rgb.r, rgb.b]).collect()
    }
}

